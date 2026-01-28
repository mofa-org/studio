//! Dora Process Manager
//!
//! Manages the lifecycle of dora-daemon and dora-coordinator processes.
//! Ensures clean startup and shutdown of all dora-related processes.

use log::{info, warn, error, debug};
use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

/// Manages dora daemon and coordinator processes
pub struct DoraProcessManager {
    daemon_child: Option<Child>,
    coordinator_child: Option<Child>,
}

impl DoraProcessManager {
    /// Create a new process manager (not started)
    pub fn new() -> Self {
        Self {
            daemon_child: None,
            coordinator_child: None,
        }
    }

    /// Start dora daemon and coordinator
    ///
    /// This will:
    /// 1. Kill any existing dora processes (multiple attempts)
    /// 2. Start dora-daemon
    /// 3. Start dora-coordinator
    /// 4. Verify they are running
    pub fn start(&mut self) -> Result<(), String> {
        info!("Starting dora processes...");

        // Step 1: Kill any existing dora processes (multiple attempts for thorough cleanup)
        info!("Cleaning up any existing dora processes...");
        for i in 1..=3 {
            debug!("Cleanup attempt {}/3", i);
            self.kill_all_processes();
            if i < 3 {
                thread::sleep(Duration::from_millis(200));
            }
        }

        // Final verification that no dora processes remain
        thread::sleep(Duration::from_millis(300));
        let pids_before = get_dora_pids();
        if !pids_before.is_empty() {
            warn!("Warning: {} dora processes still exist before start", pids_before.len());
        }

        // Step 2: Start dora-daemon
        debug!("Starting dora-daemon...");
        match Command::new("dora")
            .arg("daemon")
            .arg("--quiet")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                self.daemon_child = Some(child);
                info!("✓ dora-daemon started (PID: {})", pid);
            }
            Err(e) => {
                let msg = format!("Failed to start dora-daemon: {}", e);
                error!("{}", msg);
                return Err(msg);
            }
        }

        // Give daemon time to initialize
        thread::sleep(Duration::from_millis(700));

        // Step 3: Start dora-coordinator
        debug!("Starting dora-coordinator...");
        match Command::new("dora")
            .arg("coordinator")
            .arg("--quiet")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                self.coordinator_child = Some(child);
                info!("✓ dora-coordinator started (PID: {})", pid);
            }
            Err(e) => {
                let msg = format!("Failed to start dora-coordinator: {}", e);
                error!("{}", msg);
                // Clean up daemon if coordinator failed
                self.kill_all_processes();
                return Err(msg);
            }
        }

        // Step 4: Verify processes are running
        thread::sleep(Duration::from_millis(700));
        if !self.verify_processes_running() {
            let msg = "Dora processes failed to start or died immediately";
            error!("{}", msg);
            // Log all dora processes for debugging
            let pids = get_dora_pids();
            error!("Found {} dora processes after failed start", pids.len());
            for pid in pids {
                error!("  - PID: {}", pid);
            }
            self.kill_all_processes();
            return Err(msg.to_string());
        }

        info!("✓ Dora processes started successfully");
        Ok(())
    }

    /// Stop dora daemon and coordinator
    ///
    /// This will:
    /// 1. Stop all dataflows
    /// 2. Kill coordinator
    /// 3. Kill daemon
    /// 4. Kill any orphaned dora processes
    pub fn stop(&mut self) {
        info!("Stopping dora processes...");

        // Stop all dataflows first
        self.stop_all_dataflows();

        // Kill our managed processes
        if let Some(mut child) = self.coordinator_child.take() {
            debug!("Killing dora-coordinator (PID: {})...", child.id());
            if let Err(e) = child.kill() {
                warn!("Failed to kill dora-coordinator: {}", e);
            }
        }

        if let Some(mut child) = self.daemon_child.take() {
            debug!("Killing dora-daemon (PID: {})...", child.id());
            if let Err(e) = child.kill() {
                warn!("Failed to kill dora-daemon: {}", e);
            }
        }

        // Kill any orphaned dora processes
        thread::sleep(Duration::from_millis(200));
        self.kill_all_processes();

        info!("✓ Dora processes stopped");
    }

    /// Check if dora processes are running
    pub fn is_running(&self) -> bool {
        self.verify_processes_running()
    }

    /// Verify dora daemon and coordinator are running
    fn verify_processes_running(&self) -> bool {
        // First, check if our managed processes are still alive by checking PIDs
        let mut daemon_running = false;
        let mut coordinator_running = false;

        if let Some(ref child) = self.daemon_child {
            let pid = child.id() as u32;
            // Use sysinfo to check if process with this PID exists and is a dora process
            let mut sys = System::new_all();
            sys.refresh_processes(ProcessesToUpdate::All, true);

            if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                if let Some(name) = process.name().to_str() {
                    if name == "dora" {
                        daemon_running = true;
                        debug!("dora-daemon (PID: {}) is still running", pid);
                    }
                }
            }
        }

        if let Some(ref child) = self.coordinator_child {
            let pid = child.id() as u32;
            // Use sysinfo to check if process with this PID exists
            let mut sys = System::new_all();
            sys.refresh_processes(ProcessesToUpdate::All, true);

            if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                if let Some(name) = process.name().to_str() {
                    if name == "dora" {
                        coordinator_running = true;
                        debug!("dora-coordinator (PID: {}) is still running", pid);
                    }
                }
            }
        }

        // Also check system processes for any dora daemon/coordinator as backup
        if !daemon_running || !coordinator_running {
            debug!("Checking system processes for dora...");
            let mut sys = System::new_all();
            sys.refresh_processes(ProcessesToUpdate::All, true);

            for (pid, process) in sys.processes() {
                if let Some(name) = process.name().to_str() {
                    // Check both process name and cmd
                    let cmd = process.cmd();
                    let cmd_str: String = cmd.iter()
                        .filter_map(|s| s.to_str())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if !daemon_running && name == "dora" && cmd_str.contains("daemon") {
                        daemon_running = true;
                        debug!("Found dora-daemon process in system (PID: {})", pid);
                    }
                    if !coordinator_running && name == "dora" && cmd_str.contains("coordinator") {
                        coordinator_running = true;
                        debug!("Found dora-coordinator process in system (PID: {})", pid);
                    }
                }
            }
        }

        let running = daemon_running && coordinator_running;
        if !running {
            warn!("Dora processes not running: daemon={}, coordinator={}",
                daemon_running, coordinator_running);
        }

        running
    }

    /// Kill all dora-related processes with multiple strategies
    fn kill_all_processes(&self) {
        debug!("Killing all dora processes...");

        // Strategy 1: Try using dora stop command first (graceful)
        let _ = Command::new("dora")
            .arg("stop")
            .arg("--all")
            .output();

        thread::sleep(Duration::from_millis(100));

        // Strategy 2: Use killall on macOS/Linux (more aggressive)
        #[cfg(target_os = "macos")]
        {
            // Try killall first (more reliable than pkill on macOS)
            let _ = Command::new("killall")
                .arg("-9")
                .arg("dora")
                .output();

            thread::sleep(Duration::from_millis(50));

            // Also try pkill as backup
            let _ = Command::new("pkill")
                .arg("-9")
                .arg("-f")
                .arg("dora")
                .output();
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = Command::new("killall")
                .arg("-9")
                .arg("dora")
                .output();
        }

        thread::sleep(Duration::from_millis(100));

        // Strategy 3: Use pgrep to find PIDs and kill them individually
        if let Ok(output) = Command::new("pgrep")
            .arg("-f")
            .arg("dora")
            .output()
        {
            if let Ok(pids_str) = String::from_utf8(output.stdout) {
                for pid_str in pids_str.lines() {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        debug!("Killing dora process PID: {}", pid);
                        let _ = Command::new("kill")
                            .arg("-9")
                            .arg(pid.to_string())
                            .output();
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));

        // Strategy 4: Final verification and cleanup using sysinfo
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut killed_count = 0;
        let pids_to_kill: Vec<u32> = sys.processes()
            .iter()
            .filter(|(_, process)| {
                process.name()
                    .to_str()
                    .map(|name| name.contains("dora"))
                    .unwrap_or(false)
            })
            .map(|(pid, _)| pid.as_u32())
            .collect();

        for pid in pids_to_kill {
            debug!("Found remaining dora process (PID: {}), force killing...", pid);
            #[cfg(target_os = "macos")]
            {
                // On macOS, try both kill and killall
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .output();

                // Also try killing by process name using killall
                let _ = Command::new("killall")
                    .arg("-9")
                    .arg("dora")
                    .output();
            }

            #[cfg(not(target_os = "macos"))]
            {
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .output();
            }

            killed_count += 1;
        }

        if killed_count > 0 {
            info!("Killed {} orphaned dora processes", killed_count);
        }

        // Final verification after all killing attempts
        thread::sleep(Duration::from_millis(200));
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let remaining: Vec<_> = sys.processes()
            .iter()
            .filter(|(_, process)| {
                process.name()
                    .to_str()
                    .map(|name| name.contains("dora"))
                    .unwrap_or(false)
            })
            .collect();

        if !remaining.is_empty() {
            warn!("Warning: {} dora processes still remain after cleanup", remaining.len());
            for (pid, process) in remaining {
                warn!("  - PID: {}, Name: {:?}", pid, process.name());
            }
        } else {
            debug!("✓ All dora processes successfully killed");
        }
    }

    /// Stop all running dataflows
    fn stop_all_dataflows(&self) {
        debug!("Stopping all dataflows...");

        // Use dora list to get all dataflows
        match Command::new("dora")
            .arg("list")
            .output()
        {
            Ok(output) => {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    // Parse dataflow IDs and stop them
                    for line in stdout.lines() {
                        if line.contains("Running") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() > 0 {
                                let dataflow_id = parts[0];
                                debug!("Stopping dataflow: {}", dataflow_id);

                                let _ = Command::new("dora")
                                    .arg("stop")
                                    .arg(dataflow_id)
                                    .arg("--grace-duration")
                                    .arg("0s")
                                    .output();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to list dataflows: {}", e);
            }
        }
    }
}

impl Drop for DoraProcessManager {
    fn drop(&mut self) {
        debug!("DoraProcessManager dropping, cleaning up...");
        self.stop();
    }
}

/// Convenience function to get all dora-related PIDs
pub fn get_dora_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    for (pid, process) in sys.processes() {
        if let Some(name) = process.name().to_str() {
            if name.contains("dora") {
                pids.push(pid.as_u32());
            }
        }
    }

    pids
}

/// Check if dora command is available
pub fn is_dora_available() -> bool {
    Command::new("dora")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dora_availability() {
        let available = is_dora_available();
        println!("Dora available: {}", available);
    }

    #[test]
    fn test_get_dora_pids() {
        let pids = get_dora_pids();
        println!("Found {} dora processes", pids.len());
        for pid in pids {
            println!("  - PID: {}", pid);
        }
    }
}

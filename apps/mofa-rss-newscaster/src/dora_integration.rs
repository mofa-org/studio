//! Dora integration for RSS Newscaster
//!
//! Starts/stops the RSS dataflow and bridges UI prompts/results via
//! mofa-prompt-input dynamic node.

use crossbeam_channel::{bounded, Receiver, Sender};
use mofa_dora_bridge::{
    controller::DataflowController, dispatcher::DynamicNodeDispatcher, DoraData, SharedDoraState,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Commands sent from UI to the dora worker
#[derive(Debug, Clone)]
pub enum DoraCommand {
    StartDataflow { dataflow_path: PathBuf },
    StopDataflow,
    SendPrompt { message: String },
}

/// Events sent from dora worker to UI
#[derive(Debug, Clone)]
pub enum DoraEvent {
    DataflowStarted { dataflow_id: String },
    DataflowStopped,
    Error { message: String },
}

/// Dora integration manager (RSS Newscaster)
pub struct DoraIntegration {
    running: Arc<AtomicBool>,
    shared_state: Arc<SharedDoraState>,
    command_tx: Sender<DoraCommand>,
    event_rx: Receiver<DoraEvent>,
    worker_handle: Option<thread::JoinHandle<()>>,
    stop_tx: Option<Sender<()>>,
}

impl DoraIntegration {
    pub fn new() -> Self {
        let (command_tx, command_rx) = bounded(100);
        let (event_tx, event_rx) = bounded(100);
        let (stop_tx, stop_rx) = bounded(1);

        let running = Arc::new(AtomicBool::new(false));
        let running_clone = Arc::clone(&running);
        let shared_state = SharedDoraState::new();
        let shared_state_clone = Arc::clone(&shared_state);

        let handle = thread::spawn(move || {
            Self::run_worker(
                running_clone,
                shared_state_clone,
                command_rx,
                event_tx,
                stop_rx,
            );
        });

        Self {
            running,
            shared_state,
            command_tx,
            event_rx,
            worker_handle: Some(handle),
            stop_tx: Some(stop_tx),
        }
    }

    pub fn shared_dora_state(&self) -> &Arc<SharedDoraState> {
        &self.shared_state
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn start_dataflow(&self, dataflow_path: impl Into<PathBuf>) -> bool {
        self.command_tx
            .send(DoraCommand::StartDataflow {
                dataflow_path: dataflow_path.into(),
            })
            .is_ok()
    }

    pub fn stop_dataflow(&self) -> bool {
        self.command_tx.send(DoraCommand::StopDataflow).is_ok()
    }

    pub fn send_prompt(&self, message: impl Into<String>) -> bool {
        self.command_tx
            .send(DoraCommand::SendPrompt {
                message: message.into(),
            })
            .is_ok()
    }

    pub fn poll_events(&self) -> Vec<DoraEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    fn run_worker(
        running: Arc<AtomicBool>,
        shared_state: Arc<SharedDoraState>,
        command_rx: Receiver<DoraCommand>,
        event_tx: Sender<DoraEvent>,
        stop_rx: Receiver<()>,
    ) {
        let mut dispatcher: Option<DynamicNodeDispatcher> = None;

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            while let Ok(cmd) = command_rx.try_recv() {
                match cmd {
                    DoraCommand::StartDataflow { dataflow_path } => {
                        if running.load(Ordering::Acquire) {
                            log::warn!("Dataflow already running");
                            continue;
                        }

                        log::info!("Starting dataflow: {:?}", dataflow_path);
                        match DataflowController::new(&dataflow_path) {
                            Ok(controller) => {
                                let mut disp = DynamicNodeDispatcher::with_shared_state(
                                    controller,
                                    Arc::clone(&shared_state),
                                );
                                match disp.start() {
                                    Ok(dataflow_id) => {
                                        running.store(true, Ordering::Release);
                                        dispatcher = Some(disp);
                                        let _ = event_tx.send(DoraEvent::DataflowStarted {
                                            dataflow_id,
                                        });
                                    }
                                    Err(e) => {
                                        let message = format!("Failed to start dataflow: {}", e);
                                        log::error!("{}", message);
                                        let _ = event_tx.send(DoraEvent::Error { message });
                                    }
                                }
                            }
                            Err(e) => {
                                let message = format!("Failed to load dataflow: {}", e);
                                log::error!("{}", message);
                                let _ = event_tx.send(DoraEvent::Error { message });
                            }
                        }
                    }
                    DoraCommand::StopDataflow => {
                        if let Some(mut disp) = dispatcher.take() {
                            if let Err(e) = disp.stop() {
                                log::error!("Failed to stop dataflow: {}", e);
                            }
                        }
                        running.store(false, Ordering::Release);
                        let _ = event_tx.send(DoraEvent::DataflowStopped);
                    }
                    DoraCommand::SendPrompt { message } => {
                        if let Some(ref disp) = dispatcher {
                            if let Some(bridge) = disp.get_bridge("mofa-prompt-input") {
                                if let Err(e) =
                                    bridge.send("prompt", DoraData::Text(message.clone()))
                                {
                                    log::error!("Failed to send prompt: {}", e);
                                }
                            } else {
                                log::warn!("mofa-prompt-input bridge not found");
                            }
                        } else {
                            log::warn!("No dispatcher available to send prompt");
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

impl Drop for DoraIntegration {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

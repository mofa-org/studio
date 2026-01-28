#!/bin/bash
# Monitor Dora node logs in real-time

echo "Monitoring Dora logs..."
echo "Press Ctrl+C to stop"
echo ""

# Kill any existing dora processes first to ensure clean logs
pkill -f "dora daemon" 2>/dev/null || true
pkill -f "dora coordinator" 2>/dev/null || true
sleep 2

# Start daemon with output to file
conda run -n mofa-studio dora daemon > /tmp/dora-daemon.log 2>&1 &
DAEMON_PID=$!

# Start coordinator with output to file
conda run -n mofa-studio dora coordinator > /tmp/dora-coordinator.log 2>&1 &
COORD_PID=$!

sleep 3

echo "Dora processes started:"
echo "  Daemon PID: $DAEMON_PID"
echo "  Coordinator PID: $COORD_PID"
echo ""
echo "Now run mofa-studio in another terminal:"
echo "  cd /Users/loubicheng/project/mofa-studio"
echo "  ./run-mofa-studio.sh"
echo ""
echo "Logs will appear below:"
echo "=================================="

# Tail the logs in real-time
tail -f /tmp/dora-daemon.log /tmp/dora-coordinator.log

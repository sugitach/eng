#!/bin/bash
set -e
cd "$(dirname "$0")/.."

echo "--- Building ---"
cargo build

echo "--- Starting Daemon Agent ---"
# 出力をバックグラウンドに逃がすと見えないのでファイルへ
cargo run --bin eng-agent -- --daemon > agent_daemon.log 2>&1 &
AGENT_PID=$!
echo "Agent PID: $AGENT_PID"
sleep 3 # 起動待ち

echo "--- Starting Client Agent (should delegate) ---"
# これは --test-mode なのでUIが一瞬出て消えるはず
cargo run --bin eng-agent -- --test-mode

echo "--- Stopping Daemon Agent ---"
kill $AGENT_PID
wait $AGENT_PID || true

echo "--- Checking Agent Log ---"
cat agent_daemon.log

echo "--- Lifecycle test passed ---"

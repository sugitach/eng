#!/bin/bash
set -e

# プロジェクトのルートディレクトリに移動
cd "$(dirname "$0")/.."

echo "--- Step 1: Building project ---"
cargo build

echo "--- Step 2: Running single integration test ---"
cargo run --bin ui -- --test-mode

echo "--- Step 3: Running parallel integration test (3 instances) ---"
# 並列実行して終了を待つ
cargo run --bin ui -- --test-mode &
PID1=$!
cargo run --bin ui -- --test-mode &
PID2=$!
cargo run --bin ui -- --test-mode &
PID3=$!

wait $PID1
wait $PID2
wait $PID3

echo "--- All integration tests passed! ---"

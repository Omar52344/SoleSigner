#!/bin/bash
set -e
cd /home/omar/proyects/SoleSigner

# Load environment variables from .env
set -a
source .env
set +a

echo "Compiling stress test..."
cargo build --bin stress

echo "Starting server..."
cargo run > server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 5

echo "Running stress test..."
cargo run --bin stress

echo "Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true
echo "Done."
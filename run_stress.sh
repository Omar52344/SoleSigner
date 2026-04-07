#!/bin/bash
set -e
cd /home/omar/proyects/SoleSigner

export DATABASE_URL=postgres://test_user:test_pass@localhost:5432/test_db
export JWT_SECRET=test-jwt-secret-change-in-production
export BCRYPT_COST=10

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
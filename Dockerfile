# Stage 1: Builder
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
# Build release
# Note: In a real ONNX setup, we might need to download libonnxruntime here or rely on the crate downloading it.
RUN cargo build --release

# Stage 2: Runtime
# Using cc-debian12 for libc support (needed by Rust std and likely onnxruntime)
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/solesigner /app/solesigner

# Copy migrations if needed at runtime? No, we embedding them via sqlx::migrate! macro usually, 
# BUT sqlx::migrate! embeds them in the binary. If using cli, we need them.
# The code uses sqlx::migrate!("./migrations"), which MACRO embeds them if configured, 
# wait, sqlx::migrate! takes a path at runtime usually?
# Actually, sqlx::migrate! macro embeds them at COMPILE time. 
# So we don't need the folder at runtime if we used the macro correctly.
# Re-checking: sqlx::migrate! works at runtime by default checking the folder unless ".sqlx" is prepared or similar.
# The doc says: "The macro will check the directory... at compile time... and embed...". 
# Yes, it embeds.

# Copy .env (Optional, usually env vars passed via docker run)
# COPY .env . 

CMD ["./solesigner"]

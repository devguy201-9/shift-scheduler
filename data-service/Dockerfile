FROM rust:1.90 as builder
WORKDIR /app

# Copy workspace
COPY . .
ENV SQLX_OFFLINE=true
# Build release binary
RUN cargo build --release -p data_service

# -------- RUNTIME STAGE --------
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/data_service .

EXPOSE 8080

CMD ["./data_service"]
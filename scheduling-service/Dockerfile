FROM rust:1.90 as builder
WORKDIR /app

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release -p scheduling_service

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/scheduling_service .

EXPOSE 8081

CMD ["./scheduling_service"]
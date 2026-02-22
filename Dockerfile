FROM rust:1.93.1-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/invitebot /usr/local/bin/invitebot

ENV SURREALDB_URL=ws://surrealdb:8000
ENV SURREALDB_USER=root
ENV SURREALDB_PASS=root

CMD ["invitebot"]

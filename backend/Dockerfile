FROM rust:1-bookworm AS builder

WORKDIR /app/backend

COPY backend/Cargo.toml ./Cargo.toml
COPY backend/src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/backend/target/release/huayuan-backend /usr/local/bin/huayuan-backend

ENV HUAYUAN_BACKEND_HOST=0.0.0.0
ENV HUAYUAN_BACKEND_PORT=8787

EXPOSE 8787

CMD ["huayuan-backend"]

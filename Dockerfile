FROM rust:latest AS builder
WORKDIR /usr/src/taco
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
LABEL authors="isolume"
LABEL org.opencontainers.image.source=https://github.com/isolume/taco
LABEL org.opencontainers.image.description="taco"
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/taco /usr/local/bin/taco
CMD ["taco"]

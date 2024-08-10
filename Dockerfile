FROM rust:latest
LABEL authors="isolume"
LABEL org.opencontainers.image.source=https://github.com/isolume/taco
LABEL org.opencontainers.image.description="taco"

WORKDIR /usr/src/taco
COPY . .

RUN cargo install --path .

CMD ["taco"]
FROM rust:1.82-bookworm AS builder

WORKDIR /app

ENV CARGO_BUILD_JOBS=1

COPY . .

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libgit2-dev \
        clang \
        cmake \
        make \
    && cargo build --release --jobs 1

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libgit2-1.5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/skynet /app/skynet

ENV RUST_LOG=skynet=info

CMD ["./skynet"]

FROM rust:1.78-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/

RUN cargo build --release -p voryn-core --bin voryn-bootstrap 2>/dev/null || true
# If no bootstrap binary yet, build the library to verify compilation
RUN cargo build --release -p voryn-network

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false voryn

COPY --from=builder /build/target/release/voryn-bootstrap /usr/local/bin/ 2>/dev/null || true

USER voryn
EXPOSE 4001/tcp 4001/udp

ENTRYPOINT ["voryn-bootstrap"]
CMD ["--listen", "/ip4/0.0.0.0/tcp/4001"]

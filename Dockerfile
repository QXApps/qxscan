# qxscan — multi-stage Docker build
FROM rust:1.88-slim-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

# Need perl + make for vendored OpenSSL compilation, plus clang for native code
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev perl make && \
    rm -rf /var/lib/apt/lists/*

RUN cargo build --release && \
    strip target/release/qxscan

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/qxscan /usr/local/bin/qxscan
COPY qxscan.toml /etc/qxscan/qxscan.toml

# Symlink so qxscan finds its config in the default search path
RUN ln -s /etc/qxscan/qxscan.toml /qxscan.toml

ENTRYPOINT ["qxscan"]
CMD ["--help"]

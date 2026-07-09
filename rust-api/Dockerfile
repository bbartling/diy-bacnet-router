# syntax=docker/dockerfile:1
#
# Multi-stage build for the pure Rust axum field-bus sidecar.

FROM rust:1.96-bookworm AS builder
WORKDIR /build

# Path deps: rusty-bacnet + rusty-haystack must exist beside diy-bacnet-server in the build context.
# Build context should be the parent directory that contains all three repos, e.g.:
#   docker build -f diy-bacnet-server/rust-api/Dockerfile -t diy-bacnet-server:rust ..
COPY rusty-bacnet /build/rusty-bacnet
COPY rusty-haystack /build/rusty-haystack
COPY diy-bacnet-server/rust-api /build/diy-bacnet-server/rust-api
COPY diy-bacnet-server/config /build/diy-bacnet-server/config

WORKDIR /build/diy-bacnet-server/rust-api
RUN cargo build --release --locked 2>/dev/null || cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl iproute2 tshark \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/diy-bacnet-server/rust-api/target/release/diy-bacnet-server /usr/local/bin/diy-bacnet-server
COPY diy-bacnet-server/config /app/config
COPY diy-bacnet-server/scripts/pcap_validate.sh /app/scripts/pcap_validate.sh

ENV OPENFDD_FIELDBUS_CONFIG_DIR=/app/config
ENV OPENFDD_FIELDBUS_HTTP_HOST=0.0.0.0
ENV OPENFDD_FIELDBUS_HTTP_PORT=8080

EXPOSE 8080/tcp
EXPOSE 47808/udp

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
  CMD curl -fsS http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/diy-bacnet-server"]

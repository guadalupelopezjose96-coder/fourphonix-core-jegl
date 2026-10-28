FROM rust:1.82-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY auraintel-mtls-aggregator ./auraintel-mtls-aggregator
COPY auraintel-edge-agent ./auraintel-edge-agent
COPY auraintel-tax-settlement ./auraintel-tax-settlement

RUN cd auraintel-mtls-aggregator && cargo build --release
RUN cd auraintel-edge-agent && cargo build --release
RUN cd auraintel-tax-settlement && cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 auraintel \
    && useradd -u 10001 -g auraintel -s /sbin/nologin -M auraintel

COPY --from=builder /build/auraintel-mtls-aggregator/target/release/auraintel-mtls-aggregator /usr/local/bin/
COPY --from=builder /build/auraintel-edge-agent/target/release/auraintel-edge-agent /usr/local/bin/
COPY --from=builder /build/auraintel-tax-settlement/target/release/auraintel-tax-settlement /usr/local/bin/

USER 10001:10001
WORKDIR /app

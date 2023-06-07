FROM rust:slim-buster as builder
RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
        libssl-dev make clang llvm protobuf-compiler \
        pkg-config libz-dev zstd git; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /madara
COPY . .
RUN cargo build --release -Z sparse-registry --config net.git-fetch-with-cli=true

FROM debian:buster-slim
LABEL description="Madara, a blazing fast Starknet sequencer" \
  authors="Oak <me+madara@droak.sh>" \
  source="https://github.com/keep-starknet-strange/madara" \
  documentation="https://docs.madara.wtf/"

COPY --from=builder /madara/target/release/madara /
COPY --from=builder /madara/infra/chain-specs /chain-specs
# 9444 JSON-RPC server
# 9615 Prometheus exporter
# 30333 P2P communication
EXPOSE 9933 9944 9615 30333
ENTRYPOINT ["/madara"]

FROM rust:slim-buster as builder
RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
        libssl-dev make clang-11 g++ llvm protobuf-compiler libprotobuf-dev \
        pkg-config libz-dev zstd git build-essential; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /madara
COPY . .
RUN cargo build --release

FROM debian:buster-slim
LABEL description="Madara, a blazing fast Starknet sequencer" \
  authors="Oak <me+madara@droak.sh>" \
  source="https://github.com/keep-starknet-strange/madara" \
  documentation="https://docs.madara.zone/"

# TODO: change the way chain-specs are copied on the node
COPY --from=builder /madara/target/release/madara /madara-bin

# Making directory to store the certificate
# Install OpenSSL
RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
            curl; \
    apt-get install -y openssl; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# Create a directory for the new certificate.
RUN mkdir -p /usr/local/share/ca-certificates/extra

# Generate a new self-signed certificate.
RUN openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /usr/local/share/ca-certificates/extra/my-cert.key \
    -out /usr/local/share/ca-certificates/extra/my-cert.crt \
    -subj "/CN=localhost" \

# Update the CA certificates bundle.
RUN update-ca-certificates

HEALTHCHECK --interval=10s --timeout=30s --start-period=10s --retries=10 \
  CMD curl --request POST \
    --header "Content-Type: application/json" \
    --data '{"jsonrpc": "2.0","method": "starknet_chainId","id":1}' http://localhost:9944 || exit 1

# 9444 JSON-RPC server
# 9615 Prometheus exporter
# 30333 P2P communication
EXPOSE 9944 9615 30333
ENTRYPOINT ["/madara-bin"]

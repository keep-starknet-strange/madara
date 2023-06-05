FROM rust:slim as builder
LABEL description="Multistage Docker image for Madara" \
  image.type="builder" \
  image.authors="abdelhamid.bakhta@gmail.com" \
  image.vendor="Substrate Developer Hub" \
  image.description="Multistage Docker image for Madara" \
  image.source="https://github.com/keep-starknet-strange/madara" \
  image.documentation="https://github.com/keep-starknet-strange/madara"

RUN apt-get -y update; \
    apt-get install -y --no-install-recommends \
        libssl-dev make clang llvm protobuf-compiler \
        pkg-config libz-dev; \
    apt-get autoremove -y; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /madara
COPY . .
RUN cargo build --release

#FROM debian:slim
FROM scratch
COPY --from=builder /madara/target/release/madara /
EXPOSE 30333 9933 9944 9615
ENTRYPOINT ["/madara"]

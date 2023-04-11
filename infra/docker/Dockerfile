# This is a base image to build substrate nodes
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /madara
COPY . .
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the binary."
FROM docker.io/library/ubuntu:20.04
LABEL description="Multistage Docker image for Madara" \
  image.type="builder" \
  image.authors="abdelhamid.bakhta@gmail.com" \
  image.vendor="Substrate Developer Hub" \
  image.description="Multistage Docker image for Madara" \
  image.source="https://github.com/keep-starknet-strange/madara" \
  image.documentation="https://github.com/keep-starknet-strange/madara"

# Copy the node binary.
COPY --from=builder /madara/target/release/madara /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /node-dev node-dev && \
  mkdir -p /chain-data /node-dev/.local/share && \
  chown -R node-dev:node-dev /chain-data && \
  ln -s /chain-data /node-dev/.local/share/madara && \
  # unclutter and minimize the attack surface
  rm -rf /usr/bin /usr/sbin && \
  # check if executable works in this container
  /usr/local/bin/madara --version

USER node-dev

EXPOSE 30333 9933 9944 9615
VOLUME ["/chain-data"]

ENTRYPOINT ["/usr/local/bin/madara"]

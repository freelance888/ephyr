#
# Stage 'build-ephyr' builds Ephyr for the final stage.
#
# https://github.com/jrottenberg/ffmpeg/blob/main/docker-images/6.0/ubuntu2004/Dockerfile
FROM jrottenberg/ffmpeg:6.0-ubuntu2004 AS base

# Install build dependencies.
RUN apt-get update \
 && apt-get install -yq curl \
 && DEBIAN_FRONTEND=noninteractive apt-get install -yq pkg-config \
 && apt-get install -yq build-essential automake gcc libtool make libssl-dev protobuf-compiler

# Install Rust.
WORKDIR /tmp/rust/

ENV RUSTUP_HOME=/tmp/rust/rustup \
    CARGO_HOME=/tmp/rust/cargo \
    PATH=/tmp/rust/cargo/bin:$PATH

RUN curl -sLO https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
 && chmod +x rustup-init \
 && ./rustup-init -y --no-modify-path --profile minimal \
                  --default-toolchain stable \
 && chmod -R a+w $RUSTUP_HOME $CARGO_HOME \
 && rustup --version \
 && cargo --version \
 && rustc --version

# Install Node.js and Yarn.
RUN (curl -Ls install-node.vercel.app/16 | bash -s -- --yes) \
 && npm install --location=global yarn


#
# Builder stage for cargo-chef
#
FROM base as chef
RUN cargo install cargo-chef

#
# Planner stage for cargo-chef
#
FROM chef as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#
# Cacher stage for cargo-chef
#
FROM chef as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Ensure Yarn dependencies are available for the frontend build in build.rs
WORKDIR /app/components/restreamer/client
COPY components/restreamer/client/package.json ./
COPY components/restreamer/client/yarn.lock ./
RUN yarn install --pure-lockfile
WORKDIR /app
RUN cargo chef cook --release --recipe-path recipe.json
RUN rm -rf ./target/release/.fingerprint/ephyr-*


#
# Final builder stage
#
FROM base as builder
WORKDIR /app
COPY --from=cacher /app/target target
COPY . .

# No need to run cargo chef cook here, as dependencies are already pre-compiled
RUN cargo build -p ephyr-restreamer --bin ephyr-restreamer --release


#
# Stage 'build-srs' prepares SRS distribution for the final stage.
#
# https://github.com/ossrs/srs/releases/tag/v4.0-r5
FROM ossrs/srs:v4.0-r5 AS build-srs


#
# Stage 'runtime' creates final Docker image to use in runtime.
#
# https://github.com/jrottenberg/ffmpeg/blob/main/docker-images/6.0/ubuntu2004/Dockerfile
FROM jrottenberg/ffmpeg:6.0-ubuntu2004 AS runtime

COPY --from=build-srs /usr/local/srs/ /usr/local/srs/
COPY --from=builder /app/target/release/ephyr-restreamer \
                        /usr/local/bin/ephyr-restreamer

ENTRYPOINT  ["/usr/local/bin/ephyr-restreamer"]

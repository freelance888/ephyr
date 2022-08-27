#
# Stage 'build-ephyr' builds Ephyr for the final stage.
#

# https://github.com/jrottenberg/ffmpeg/blob/master/docker-images/5.1/ubuntu2004/Dockerfile
FROM jrottenberg/ffmpeg:5.1-ubuntu2004 AS build-ephyr



# Install build dependencies.
RUN apt-get update \
 && apt-get install -yq curl \
 && DEBIAN_FRONTEND=noninteractive apt-get install -yq pkg-config \
 && apt-get install -yq build-essential automake gcc libtool make libssl-dev

# Intall gstreamer.
RUN apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev


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


# First, build all the dependencies to cache them in a separate Docker layer and
# avoid recompilation each time project sources are changed.
WORKDIR /tmp/ephyr/

COPY common/api/allatra-video/Cargo.toml ./common/api/allatra-video/
COPY common/log/Cargo.toml ./common/log/
COPY common/serde/Cargo.toml ./common/serde/
COPY components/restreamer/Cargo.toml ./components/restreamer/
COPY components/vod-meta-server/Cargo.toml ./components/vod-meta-server/
COPY Cargo.toml Cargo.lock ./

COPY components/restreamer/client.graphql.schema.json ./components/restreamer/


RUN mkdir -p common/api/allatra-video/src/ \
 && touch common/api/allatra-video/src/lib.rs \
 && mkdir -p common/log/src/ \
 && touch common/log/src/lib.rs \
 && mkdir -p common/serde/src/ \
 && touch common/serde/src/lib.rs \
 && mkdir -p components/restreamer/src/ \
 && touch components/restreamer/src/lib.rs components/restreamer/src/main.rs \
 && mkdir -p components/vod-meta-server/src/ \
 && touch components/vod-meta-server/src/lib.rs

RUN cargo build -p ephyr-restreamer --lib --release


# Then, prepare Yarn dependencies to cache them in a separate Docker layer and
# avoid recompilation each time project sources are changed.
WORKDIR /tmp/ephyr/components/restreamer/

COPY components/restreamer/client/.yarnrc \
     components/restreamer/client/package.json \
     components/restreamer/client/yarn.lock \
     ./

RUN yarn install --pure-lockfile


# Finally, build the whole project.
WORKDIR /tmp/ephyr/

RUN rm -rf ./target/release/.fingerprint/ephyr-*

COPY common/log/ ./common/log/
COPY components/restreamer/ ./components/restreamer/

RUN cargo build -p ephyr-restreamer --bin ephyr-restreamer --release




#
# Stage 'build-srs' prepares SRS distribution for the final stage.
#

# https://github.com/ossrs/srs/releases/tag/v4.0-r1
FROM ossrs/srs:v4.0-r1 AS build-srs




#
# Stage 'runtime' creates final Docker image to use in runtime.
#

# https://github.com/jrottenberg/ffmpeg/blob/master/docker-images/5.1/ubuntu2004/Dockerfile
FROM jrottenberg/ffmpeg:5.1-ubuntu2004 AS runtime

COPY --from=build-srs /usr/local/srs/ /usr/local/srs/

COPY --from=build-ephyr /tmp/ephyr/target/release/ephyr-restreamer \
                        /usr/local/bin/ephyr-restreamer

ENTRYPOINT  ["/usr/local/bin/ephyr-restreamer"]

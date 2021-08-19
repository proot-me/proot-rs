# Note that since cross needs the rust toolchain for *-x86_64-unknown-linux-gnu, 
# so it is better to use a glibc-based system image such as buster, rather than 
# alpine.
FROM rust:buster as build

RUN apt-get update && \
    apt-get install -y \
            bash \
            bats \
            curl \
            gcc \
            shellcheck

# Install docker
RUN apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add - && \
    echo "deb [arch=amd64] https://download.docker.com/linux/debian buster stable" | tee /etc/apt/sources.list.d/docker.list && \
    apt-get update && \
    apt-get install -y docker-ce docker-ce-cli containerd.io

# Install cargo-make
RUN curl -O -L https://github.com/sagiegurari/cargo-make/releases/download/0.35.0/cargo-make-v0.35.0-x86_64-unknown-linux-musl.zip && \
    unzip -p cargo-make-v0.35.0-x86_64-unknown-linux-musl.zip cargo-make-v0.35.0-x86_64-unknown-linux-musl/cargo-make > "${CARGO_HOME}/bin/cargo-make" && \
    chmod +x "${CARGO_HOME}/bin/cargo-make" && \
    rm cargo-make-v0.35.0-x86_64-unknown-linux-musl.zip

# Install cross
RUN cargo install --git="https://github.com/rust-embedded/cross.git" --branch="master" cross

# Install targets
RUN rustup toolchain install nightly-2021-03-24 && \
    rustup +nightly-2021-03-24 target add x86_64-unknown-linux-musl && \
    rustup +nightly-2021-03-24 target add x86_64-unknown-linux-gnu && \
    rustup +nightly-2021-03-24 target add x86_64-linux-android && \
    rustup +nightly-2021-03-24 target add i686-unknown-linux-musl && \
    rustup +nightly-2021-03-24 target add i686-unknown-linux-gnu && \
    rustup +nightly-2021-03-24 target add i686-linux-android && \
    rustup +nightly-2021-03-24 target add armv7-unknown-linux-musleabihf && \
    rustup +nightly-2021-03-24 target add armv7-unknown-linux-gnueabihf && \
    rustup +nightly-2021-03-24 target add arm-linux-androideabi && \
    rustup +nightly-2021-03-24 target add aarch64-unknown-linux-musl && \
    rustup +nightly-2021-03-24 target add aarch64-unknown-linux-gnu && \
    rustup +nightly-2021-03-24 target add aarch64-linux-android

WORKDIR /usr/src/proot-rs
COPY . /usr/src/proot-rs

ENV CROSS_DOCKER_IN_DOCKER=true

CMD ["cargo", "make", "build"]


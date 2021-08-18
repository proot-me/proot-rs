FROM runmymind/docker-android-sdk:alpine-standalone as build

RUN apk update && \
    apk add --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing \
            bash \
            bats \
            curl \
            gcc \
            rustup \
            shellcheck \
            openssl-dev \
            musl-dev

ENV PATH "/root/.cargo/bin/:/opt/android-sdk-linux/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin:${PATH}"

RUN rustup-init -y && \
    rustup toolchain install stable && \
    cargo +stable install --force cargo-make && \
    cargo install cross && \
    rustup toolchain install nightly-2021-03-24 && \
    rustup +nightly-2021-03-24 target add arm-linux-androideabi

WORKDIR /usr/src/proot-rs
COPY . /usr/src/proot-rs

CMD ["cargo", "make", "build"]


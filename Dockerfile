FROM rust:alpine as build

RUN apk update && \
    apk add bash \
            bats \
            curl \
            shellcheck \
            openssl-dev \
            musl-dev

RUN cargo install --force cargo-make

WORKDIR /usr/src/proot-rs
COPY . /usr/src/proot-rs

CMD ["cargo", "make", "build"]


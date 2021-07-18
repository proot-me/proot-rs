FROM rust:buster

# Install bats
ARG BATS_VERSION=1.3.0
WORKDIR /opt
RUN wget -O bats-core.tar.gz https://github.com/bats-core/bats-core/archive/refs/tags/v${BATS_VERSION}.tar.gz && \
    tar xf bats-core.tar.gz && \
    rm bats-core.tar.gz && \
    ln -s /opt/bats-core-${BATS_VERSION}/bin/bats /usr/local/bin/bats


# Copy source code into image
WORKDIR /code
COPY . /code

# Build new rootfs into "/rootfs"
RUN bash /code/scripts/mkrootfs.sh -d /rootfs
ENV PROOT_TEST_ROOTFS="/rootfs"


CMD ["sh", "-c", "cargo build && bats -r tests;"]
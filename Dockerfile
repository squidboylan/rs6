# Usage
#
# First, build it:
#
# docker build -t rsv6 .
#
# You'll want to mount the root of the repository into the container
# as /xv6
#
# For example, after `cd`ing into the repo root:
#
# docker run --rm -it -v $PWD:/xv6 rsv6:latest

FROM rust:1.46.0-alpine

RUN apk add --update \
    alpine-sdk \
    bash \
    nasm \
    qemu-system-i386
COPY rust-toolchain /rust-toolchain
RUN rustup toolchain install $(cat /rust-toolchain)
RUN rustup component add rust-src
WORKDIR /xv6
CMD ["/bin/bash"]

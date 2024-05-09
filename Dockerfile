FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf:main

ARG CROSS_DEB_ARCH=""
ARG CROSS_CMD

RUN eval "${CROSS_CMD}"

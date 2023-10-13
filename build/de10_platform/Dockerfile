# syntax = docker/dockerfile:experimental
################################################################################
## Project       : MiSTer FPGA
## Module        : Native ARMv7 C/C++ Toolchain for MiSTer HPS
## Purpose       : Provide easy toolchain to build applications, kernel, etc.
## Author        : Marcus Andrade <Boogermann>
## Author        : Hans Larsen <hans.larsen@gmail.com>
## Creation Date : 20191231
################################################################################
## Copyright (c) 2019 Marcus Andrade
##
## Permission is hereby granted, free of charge, to any person obtaining a copy
## of this software and associated documentation files (the "Software"), to deal
## in the Software without restriction, including without limitation the rights
## to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
## copies of the Software, and to permit persons to whom the Software is
## furnished to do so, subject to the following conditions:
##
## The above copyright notice and this permission notice shall be included in
## all copies or substantial portions of the Software.
################################################################################
## Change History [date(YYYYMMDD) Who: What]
##  20230726 HL:  Moved to Debian 11 (Bullseye) and the MiSTer repo itself.
##  20200109 MJA: moved sdl libraries to toolchain-sdl
##  20191231 MJA: first release
################################################################################
FROM arm32v7/debian:bullseye-slim
MAINTAINER Marcus Andrade "marcus@misterkun.io"
MAINTAINER Hans Larsen "hans.larsen@gmail.com"

LABEL org.opencontainers.image.authors="Marcus Andrade <marcus@misterkun.io>, Hans Larsen <hans.larsen@gmail.com>"
LABEL org.opencontainers.image.title="MiSTer FPGA Toolchain"
LABEL org.opencontainers.image.description="ARMv7 C/C++ Toolchain for MiSTer FPGA"

# Environment Variable
ENV DEBIAN_FRONTEND=noninteractive
ENV MISTER_ROOT=/mister
ENV MISTER_SD=/media/fat

# Switches for Optimization
ENV CFLAGS="-mcpu=cortex-a9 -mtune=cortex-a9 -mfloat-abi=hard -march=armv7-a -mfpu=neon"
# Add CXXFLAGS
ENV CXXFLAGS="-mcpu=cortex-a9 -mtune=cortex-a9 -mfloat-abi=hard -march=armv7-a -mfpu=neon"
ENV MAKEFLAGS="-j$(nproc)"

# Core Developement Packages
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
        autoconf \
        automake \
        bash-completion \
        bison \
        build-essential \
        ca-certificates \
        checkinstall \
        curl \
        flex \
        gdb \
        git \
        iputils-ping \
        less \
        libc-dev \
        libcurl4-openssl-dev \
        libdrm-dev \
        libelf-dev \
        libevdev-dev \
        libfreetype6-dev \
        libgbm-dev \
        libgmp3-dev \
        libinput-dev \
        libjsoncpp-dev \
        liblz4-tool \
        libmpc-dev \
        libmpfr-dev \
        libncurses5-dev \
        libpcap-dev \
        libreadline-dev \
        libsqlite3-dev \
        libssl-dev \
        libtcmalloc-minimal4 \
        libtool \
        libtool-bin \
        libusb-dev \
        libusb-1.0-0-dev \
        locales \
        mlocate \
        nano \
        nasm \
        openssh-client \
        p7zip \
        python3-pip \
        pkg-config \
        software-properties-common \
        sshpass \
        sudo \
        tcl \
        texinfo \
        tree \
        unzip \
        wget \
        xz-utils \
        zlib1g-dev

# Set System Locales
RUN echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen && locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

# Base Directories
RUN mkdir -p "${MISTER_ROOT}" "${MISTER_SD}" && \
	chmod -R 777 "$MISTER_ROOT" && \
	chmod -R 777 "$MISTER_SD"

# Fix some core dependencies without rebuilding all layers
RUN apt-get update && \
    apt-get upgrade -y && \
	apt-get install -y --no-install-recommends \
        cmake \
        gettext \
        libglib2.0-dev \
        libltdl-dev \
        autopoint \
        libudev-dev \
        udev \
        libdbus-1-dev \
        libdbus-glib-1-dev \
        intltool \
        autogen \
        shtool \
    && \
	rm -r /var/lib/apt/lists/*


# Build SDL2 from source.
RUN cd && \
    git clone https://github.com/libsdl-org/SDL && \
    cd SDL && \
    git checkout f2ae00c1ad5194586bacbd7b374422bd1d47c81a && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_BUILD_TYPE=Release && \
    cmake --build . --config Release --parallel && \
    cmake --install . --config Release

# TODO: Rust has an issue when building in a 32-bit container from a 64-bit host. Figure out a fix.
#       The issue prevents us from using GIT as dependencies.
# Install rustup and the Rust toolchain
RUN cd && \
    curl https://sh.rustup.rs -sSf > install.sh && \
    sh install.sh -y
ENV PATH="$HOME/.cargo/bin:$PATH"
RUN echo "[net] \n\
          git-fetch-with-cli = true" >> "$HOME/.cargo/config"
#ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Set Volume and Workdir
VOLUME /mister
WORKDIR /mister

# Bash Settings
RUN echo "export HISTTIMEFORMAT='%d/%m/%y %T '" >> ~/.bashrc && \
    echo "export PS1='\[\e[0;36m\]\u\[\e[0m\]@\[\e[0;33m\]\h\[\e[0m\]:\[\e[0;35m\]\w\[\e[0m\]\$ '" >> ~/.bashrc && \
    echo "alias ll='ls -lah'" >> ~/.bashrc && \
    echo "alias ls='ls --color=auto'" >> ~/.bashrc

# Metadata Params
ARG BUILD_DATE
ARG VCS_REF
ARG BUILD_VERSION

LABEL   org.label-schema.schema-version="1.0" \
        org.label-schema.vendor="Main_MiSTer" \
        org.label-schema.name="Main_MiSTer/toolchain" \
        org.label-schema.description="ARMv7 C/C++ Toolchain for MiSTer FPGA" \
        org.label-schema.url="https://github.com/hansl/Main_MiSTer" \
        org.label-schema.vcs-url="https://github.com/hansl/Main_MiSTer.git" \
        org.label-schema.build-date=$BUILD_DATE \
        org.label-schema.vcs-ref=$VCS_REF \
        org.label-schema.version=$BUILD_VERSION


CMD ["/bin/bash"]

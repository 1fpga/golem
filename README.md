# GoLEm FPGA Firmware

This repo is the main code repo for the GoLEm FPGA Firmware and its companion libraries and binaries.

## Crates

This repo is a monorepo, containing multiple crates.
The main crate is `GoLEm_firmware`, which is the actual firmware for GoLEm.
It is meant as a drop-in replacement for MiSTer.

This repo also contains the twin packages `senior` and `junior`, which are the client and server respectively.
They are meant to be used together to replace the MiSTer firmware into a client/server architecture to help debug and develop cores and firmware features that don't require user interface.
Basically, running `junior` on the DE10-Nano, it starts a webserver and does not provide an interactive interface.
The webserver provides a full REST API to interact with the FPGA, and the `senior` client can be used as a client CLI to interact with it (though not needed).
## FAQ

### What is this?

GoLEm wants to be a replacement for the MiSTer Firmware, achieving the same emulation capabilities, but with a better codebase (more maintainable) and an easier, user-focused interface.

From MiSTer's [wiki](https://github.com/MiSTer-devel/Wiki_MiSTer/wiki):

> MiSTer is an open project that aims to recreate various classic computers, game consoles and arcade machines, using modern hardware.
> It allows software and game images to run as they would on original hardware, using peripherals such as mice, keyboards, joysticks and other game controllers.

### What are you doing to it?

The MiSTer code is currently coded in legacy C and C++.
It is hard to maintain, hard to build, and hard to contribute to.

GoLEm is written in easier-to-maintain (but still Open Source) Rust.
The design of the application has been made from top down to enable contributions and maintenance.
It is easier to read this code.

This is also an opportunity to improve the user experience greatly.
For example,

### How can I help?

Try it, get up to speed with the MiSTer project itself, and get ready to contribute when the time comes.

## Development

There are currently two parts to this repo; a Rust library that is then linked with the C++ code.

### Structure

The current structure is still in flux, but essentially the `rust` code is in the `src` directory, while the C++ code is in the root.
C++ dependencies are located in the `lib/` folder, and the `support/` folder contains various code necessary to run the cores.

### Prerequisites

To build the `rust` library portion of this code, you'll need to install the `rust` toolchain.
The easiest way to do this is to use `rustup`.
Instructions can be found [here](https://rustup.rs/).

### Building MiSTer for DE10-Nano

To build the DE10 target executable, you will also need [Docker](https://www.docker.com) and [`cross`](https://https://github.com/cross-rs/cross?tab=readme-ov-file) installed.

Assuming you have Docker installed, you can install `cross` with the following command:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Then, you can build the firmware with the following command:

```bash
cross build --bin golem --target=armv7-unknown-linux-gnueabihf --no-default-features --features=platform_de10 --release
```

If everything goes well, this will output the executable in `./target/armv7-unknown-linux-gnueabihf/release/golem`. Simply copy that binary to your device and execute it.

The following commands can help:

```bash
ssh root@$MISTER_IP 'killall MiSTer GoLEm_firmware' # Make sure MiSTer (and GoLEm) is not running
scp ./target/armv7-unknown-linux-gnueabihf/release/golem root@$MISTER_IP:/media/fat/GoLEm_firmware # Copy the binary to the device
ssh root@$MISTER_IP 'sync; PATH=/media/fat:$PATH; GoLEm_firmware' # Restart the firmware
```

Running `GoLEm_firmware --help` will show you the available CLI options.

### Desktop Executable

There is a Desktop version of this (that does not support Cores), which can be ran locally in debug mode with:

```bash
cargo run --bin golem
```

This version should help develop some features that don't require an FPGA (like menus and configs).

### Tests

Tests can be run with `cargo test` as you would.

# Contributing

This repo is not the main fork of the MiSTer firmware.
If you want to contribute to MiSTer itself, please go to the [MiSTer repo](https://github.com/MiSTer-devel/Main_MiSTer/).

You can help a lot by testing this firmware and report bugs.

To contribute, please fork this repo, make your changes, and submit a PR.
Most PRs should be approved right away, but some may require discussion.

Make sure you follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) when contributing.
We use the Rust CoC currently because it is the most complete and well thought out CoC we could find.
We might fork it locally in the future.

Thank you for understanding.


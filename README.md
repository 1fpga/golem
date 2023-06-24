# Main_MiSTer Main Firmware

This repo serves as the home for the MiSTer Main binaries and the Wiki.

## Wiki

For the purposes of getting google to crawl the wiki, here's a link to the (not for humans) [crawlable wiki](https://github-wiki-see.page/m/MiSTer-devel/Wiki_MiSTer/wiki)

If you're a human looking for the wiki, that's [here](https://github.com/MiSTer-devel/Wiki_MiSTer/wiki)

## Development
There are currently two parts to this repo; a Rust library that is then linked with the C++ code.

### Structure
The current structure is still in flux, but essentially the `rust` code is in the `src` directory, while the C++ code is in the root.
C++ dependencies are located in the `lib/` folder, and the `support/` folder contains various code necessary to run the cores.

### Prerequisites
To build the `rust` library portion of this code, you'll need to install the `rust` toolchain.
The easiest way to do this is to use `rustup`.
Instructions can be found [here](https://rustup.rs/).

The easiest way to build the `MiSTer` binary is to use the Docker container provided by `misterkun` [here](https://hub.docker.com/r/misterkun/toolchain).

### Building

To build the `MiSTer` binary, run the following commands:

```bash
cargo build --release
docker run -it --rm -v $PWD:/mister misterkun/toolchain make BASE=arm-linux-gnueabihf
```

If everything goes well, you should have a `MiSTer` file in the root directory of this repo.
Simply copy that binary to your device and execute it.
The following commands can help:

```bash
ssh root@$MISTER_IP 'killall MiSTer' # Make sure MiSTer is not running
scp MiSTer root@$MISTER_IP:/media/fat/ # Copy the binary to the device
ssh root@$MISTER_IP 'sync; PATH=/media/fat:$PATH; MiSTer' # Restart the firmware
```

# Contributing
This repo is not the main fork of the MiSTer firmware.
It is not ready to receive contributions.
When that changes, this section will be updated.

Thank you for understanding.


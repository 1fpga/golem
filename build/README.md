# Building MiSTer-rs
This folder has specific instructions on how the build system works.

## Desktop
To build a debug executable on the desktop, simply use `cargo build`.
This will build the project and place the binary in `target/debug/mister`.

To build a release executable on the desktop, use `cargo build --release`.

## DE10-Nano
To build the DE10-Nano platform version of this executable, you will need to have Docker installed.

First, build the image in this repository:

```bash
docker build -t mister-toolchain build/de10_platform
```

First, configure the pkg-config environment variables to point to the correct locations:

```bash
export PKG_CONFIG_PATH=$PWD/lib/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=$PWD/lib/sysroot/
```

And then, set the `platform_de10` feature, disable the default features, and set the target architecture properly:

```bash
cargo build --lib --target=armv7-unknown-linux-gnueabihf --no-default-features --features=platform_de10 --release
```

Finally, use the docker image to build the CPP code and link it all together:

```bash
docker run -it --rm -v $PWD:/mister mister-toolchain make BASE=arm-linux-gnueabihf
```

This should give you a `MiSTer` executable in the root of your project.
Simply copy that executable to your MiSTer install (in `/media/fat`) and re-run it to test it out.

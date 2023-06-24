# Main_MiSTer Main Firmware

This repo serves as the home for the MiSTer Main binaries and the Wiki.

## FAQ

### What is this?
From the [wiki](https://github.com/MiSTer-devel/Wiki_MiSTer/wiki):

> MiSTer is an open project that aims to recreate various classic computers, game consoles and arcade machines, using modern hardware. It allows software and game images to run as they would on original hardware, using peripherals such as mice, keyboards, joysticks and other game controllers.

This specific repo is the main firmware that runs on the MiSTer hardware, responsible to initialize and coordinate with the FPGA and the various cores.

### What are you doing to it?
The MiSTer code is currently coded in C++.
This fork is an attempt to rewrite the main firmware in Rust.

### Why?
Multiple reasons:

1. Learning (the "Why not?" answer).
   It's a good occasion for me to learn the source code of the MiSTer firmware, which I've been trying to read for a while.
2. The current codebase is a bit of a mess.
   It's hard to start, build, follow, and make changes to it.
   Installing the toolchain and building the code is a pain and will take likely a few hours to get going.
   Which makes it hard to contribute to the project, as a general rule.
   In the case of Rust, the toolchain and arch support is generally better, and the barrier of entry is lower.
   I'm hopeful that by the end of my first step, people will only have to do `cargo build` to build the entire firmware.
3. Rust is a safer language than C++.
   It's easier to write safe code in Rust than in C++.
   For a firmware that runs at a very low level, this is crucial as it can prevent a lot of bugs.
4. Rust is also a more modern language than C++.
   Compilers optimize Rust better (resulting in faster code which matters on embedded systems with limited resources), and it is easier to contribute features to it.
   The code in general will be reduced by a lot (I predict more than half).
5. Better ecosystem.
   Rust has a lot of packages (named crates) available to it which saves us development time and can be imported and reused out of the box.

### Did you say first step?
Yeah, I've separated my work into multiple milestones:

1. Move to Rust.
   Do not care about the code quality, just make sure it works and is as close to original code as possible.
2. Refactor code to be more Rust-like.
   This includes using more idiomatic Rust, and using more of the standard library.
   This should improve readability and maintainability of the code, significantly.
3. Add support for proper debugging tools.
   This includes adding logging, and adding support for `gdb` debugging (including remote).
   This should make it easier to debug in general and should make it possible not to rely on "printf" debugging.
   Another improvement I could see would be the ability to mock firmware, which would allow running and testing the firmware on a desktop, without an FPGA.
4. General code hygiene and improvements.
   I don't have a clear list after that, but basically a mix of (in no particular order):
   1. Adding Tests and CI/CD support.
      Proper nightly builds.
   2. Adding Documentation.
   3. Refactoring the GUI to be more newbie friendly.
      As inspiration the Analogue OS is a good example of a newbie friendly GUI.
   4. Move the core support code to a separate plugin model, e.g. WASM.
   5. Improving the general UX of the firmware.
      I have had my shares of frustrations with the menus and options and I believe it will be easier to improve on those in Rust than in the current codebase.
   6. Anything else that comes to my mind.

### Okay I'm sold, but why you?
I have over 4 years of Rust experience at this point, I've been working on embedded systems before, and I've already ported a codebase from C++ to Rust.
Basically, I know what I'm doing, as I've done it before.

### What's your timeline?
I'm hoping I can be done with the first milestone above by end of July (I have obligations outside of this project).
Since I'm fulltime, as soon as summer ends I should be able to dedicate more time to this project and get it real quick through the next steps.

### How can I help?
Right now, not much can be done.
Try it, get up to speed with the MiSTer project itself, and get ready to contribute when the time comes.
Before milestone 2 is done, I don't want to accept any PRs, as I'm going to be rewriting a lot of the code anyway.

But the main wiki and documentation can always be improved on.

### Are you going to maintain this fork?
I hope not.
My goal would be to get this fork merged back into the main repo and have contributors continue to work on it from there.

Time will tell if this is going to work.

### Anything else?
I stream these efforts often; you can find me [here](https://www.twitch.tv/hanslatwork).

I am also on a bunch of discords: MiSTer, FPGaming, Rust, and others.

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


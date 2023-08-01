# Startup Sequence
The firmware exposes a `main()` function that gets called by the system:
- On the DE10 platform, it's built and linked using the `Makefile`.
- On desktop, it's in `main.rs`.
- In tests, it's also in `lib.rs` but is not used.

Only 1 platform can be selected.

The `main()` function calls the `main_inner::main()` which is actually responsible for starting the firmware in all three platforms.

It transfers control to the platform manager which is unique code per platform.

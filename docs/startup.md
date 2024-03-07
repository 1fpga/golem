# Startup Sequence

The firmware exposes a `main()` function that gets called by the system:

- On the DE10 platform, it's built and linked using the `Makefile`.
- On desktop, it's in `main.rs`.
- In tests, it's also in `lib.rs` but is not used.

Only 1 platform can be selected.

It transfers control to the platform manager which is unique code per platform.

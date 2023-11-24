# Developing Ares

## Nix
Ares uses a Nix developer shell to set up the environment for rust builds. Please [Install Nix](https://nixos.org/download#download-nix).
With Nix installed, you can run

```bash
nix develop
```

in `rust/` or any subdirectory, and you will be dropped into a BASH shell with the build environment set up. This will provide proper versions of non-rust dependencies, as well as the rust environment.

## Rust

To build Ares, start a nix development shell as above. Within the shell, in the `rust/ares` directory, you can run:

```bash
cargo build
```

to build the Ares executable. This will place the built executable at `target/debug/ares` under the `rust/ares` directory.

Ares is made to run as an urbit "serf", meaning it is intended to be invoked by a "king" which sends it commands and performs side-effects specified by its output. We use the vere king. Special instructions for building the vere king to invoke Ares are forthcoming.

To watch rust and check for errors, run

```bash
cargo watch --clear
```

Until terminated with ctrl-c, this will rebuild Ares library on any change to the underlying source files and report any warnings and errors. It will *not* produce the executable. You must run the build command above to rebuild the executable.

## Hoon

The Nock analysis and lowering for Ares is written in Hoon, and lives at `hoon/codegen.` It is meant to be jammed and included in the Ares binary. (See [`src/load.rs`](rust/ares/src/load.rs) in the Rust sources for details.)

If the hoon source has been synced to a desk, e.g. `sandbox`, on a fakezod, then the build generator can be invoked as:

```
.cg/jam +sandbox!cg-make
```

This will build the Hoon standard library and the Ares Nock analysis as a "trap" meant to be run by Ares. The jammed output can be found at `<fakezod-pier>/.urb/put/cg.jam`, and should be copied to the `rust/ares/bin` directory, from whence the rust build will include it in the executable.

Instructions on testing the analysis in a fakezod are forthcoming.

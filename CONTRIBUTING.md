# Contributing Guide

Thank you so much for showing an interest in contributing to rusty-merge! I am excited to see what you have to add :)

## 🦀 Rust Toolchain

rusty-merge uses the nightly distribution of the rust toolchain. You can get on the nightly edition of rustfmt, the only toolchain component used by rusty-merge, by running the following command:

```bash
rustup component add rustfmt --toolchain nightly
```

## 🏗️ Build System

rusty-merge uses [cargo-make](https://github.com/sagiegurari/cargo-make) for all build scripts. Please run the following command to install cargo-make:

```bash
cargo install --force cargo-make
```

Once you have cargo-make installed you can run `cargo make TASK_NAME` to run a certain task. All tasks are prefixed with `tasks.` and are stored in the [Makefile.toml](Makefile.toml) file. So if you wanted to build the development binary for example you would run the following command:

```bash
cargo make build-rust-dev
```

## 🧪 Linters

rusty-merge only uses two linters: [hadolint](https://github.com/hadolint/hadolint) and [rustfmt](https://github.com/rust-lang/rustfmt). hadolint will lint the dockerfiles stored in [docker/](docker/) and rustfmt will lint the source code in [src/](src/). Please install hadolint using your system's package manager and rustfmt with the toolchain command provided in the [🦀 Rust Toolchain section](#-rust-toolchain).

That's all, good luck with your contribution, and please make an issue if anything doesn't make sense.

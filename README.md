# Cargo GCCRS

Gaining support for cargo via rustc-wrapper

This project is *not* usable yet.

## Description

The aim of this project is to allow the use of the [gccrs](https://github.com/Rust-GCC/gccrs)
compiler alongside the [cargo buildsystem](https://github.com/rust-lang/cargo).

## Setup

Since the project does not contain any releases yet, you can install it from github using
the following command:

```sh
> cargo install --git https://github.com/Rust-GCC/cargo-gccrs
```

## Usage

The goal is to provide an alternative to the classical subcommands used when working on
Rust project, such as `build`, `run` or `test`. You should simply use `cargo gccrs` instead
of `cargo` if you wish to execute commands using `gccrs` instead of `rustc`.

## [Code of Conduct](CODE_OF_CONDUCT.md)

This repository adopts the [Contributor Covenant Code of
Conduct](https://www.contributor-covenant.org/version/1/4/code-of-conduct/).

## Contributing

{Contribution instructions}

## License

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
https://opensource.org/licenses/MIT>, at your option. Files in the project may
not be copied, modified, or distributed except according to those terms.

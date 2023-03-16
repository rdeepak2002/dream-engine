# Dream Engine [WIP]

![CI Status](https://github.com/rdeepak2002/dream-rs/actions/workflows/ci.yml/badge.svg?branch=main) [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

<p align="center">
  <a href="https://github.com/rdeepak2002/dream-rs">
    <img src="doc/image/logo.png" height="162" alt="Dream Engine Logo">
  </a>
</p>

## Author

Deepak Ramalingam

## About

Re-creation of [Dream Engine](https://github.com/rdeepak2002/dream) in Rust. 

## Requirements

- [rust](https://www.rust-lang.org/tools/install) (Version 1.68.0)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (Version 0.10.3)

- wasm32-unknown-unknown target for Rust
    - Installation command:
```shell
rustup target add wasm32-unknown-unknown
```

## Get Started (Desktop)

```shell
cargo run
```

## Get Started (Web)

```shell
./tools/build-web.sh
```

Serve ``web/index.html`` from ``web`` folder

## Screenshots

### MacOS Desktop Build

![desktop](doc/image/screenshot_0.png)

### Web Assembly (Browser) Build

![web](doc/image/screenshot_1.png)

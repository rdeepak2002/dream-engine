# Development

## Requirements

- [rust](https://www.rust-lang.org/tools/install) (Version 1.72.0-nightly)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (Version 0.10.3)

- wasm32-unknown-unknown target for Rust
    - Installation command:

```shell
rustup target add wasm32-unknown-unknown
```

## Build for Desktop

Run the following command in ``crates/dream-runner``

```shell
cargo run +nightly --package dream-runner --bin dream-runner  --release
```

## Build for Web

Run the following command in ``crates/dream-runner``

```shell
# build project
rustup run nightly-2022-12-12 wasm-pack build --target web --out-dir ../../web/build --release

# serve it on the web by starting the server
cd ../../web
npm i
npm run start
```

Visit [http://localhost:3000](http://localhost:3000) on the latest version of Chrome to view the application

## Run Tests

```shell
cargo test --workspace
```

## Deployment

Please refer to [DEPLOYMENT.md](deployment.md)

## Troubleshooting

### ``error[E0554]: #![feature] may not be used on the stable release channel``

Recommended solution: ``cargo clean``

Other
solutions: [https://stackoverflow.com/questions/53136717/errore0554-feature-may-not-be-used-on-the-stable-release-channel-couldnt](https://stackoverflow.com/questions/53136717/errore0554-feature-may-not-be-used-on-the-stable-release-channel-couldnt)

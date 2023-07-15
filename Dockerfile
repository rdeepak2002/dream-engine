FROM ubuntu:16.04
RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install g++ valgrind -y

FROM rustlang/rust:nightly
WORKDIR /usr/src/myapp
COPY . .
RUN rustup toolchain install nightly-2022-12-12-aarch64-unknown-linux-gnu
RUN rustup component add rust-src --toolchain nightly-2022-12-12-aarch64-unknown-linux-gnu
RUN cargo install --path ./crates/dream-runner --target aarch64-unknown-linux-gnu
RUN cargo run
CMD ["myapp"]
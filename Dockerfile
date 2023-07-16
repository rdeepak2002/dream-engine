FROM ubuntu:16.04
RUN apt-get update
RUN apt-get install -y \
    build-essential \
    curl
RUN apt-get upgrade -y
RUN apt-get install g++ valgrind -y
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

#FROM rustlang/rust:nightly
#WORKDIR /usr/src/myapp
COPY . .
RUN rustup toolchain install nightly-2022-12-12-aarch64-unknown-linux-gnu
RUN rustup component add rust-src --toolchain nightly-2022-12-12-aarch64-unknown-linux-gnu
#RUN cargo install cargo-valgrind
RUN cargo install --path ./crates/dream-runner --target aarch64-unknown-linux-gnu
#RUN cargo valgrind run
#RUN cargo run
CMD ["myapp"]
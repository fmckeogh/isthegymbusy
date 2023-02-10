FROM rust AS builder
WORKDIR /tmp/isthegymbusy

# prepare toolchain
RUN rustup target add x86_64-unknown-linux-musl

# add musl tools
RUN apt-get update && apt-get install musl-tools clang llvm -y

# build dependencies
RUN cargo init --bin .
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release --target x86_64-unknown-linux-musl

# build app
COPY src src
RUN touch src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
ENV RUST_LOG=
ENV ADDRESS=
COPY --from=builder /tmp/isthegymbusy/target/x86_64-unknown-linux-musl/release/isthegymbusy .
ENTRYPOINT ["./isthegymbusy"]

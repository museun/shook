FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /usr/src/shook

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/shook/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin shook

FROM debian:bullseye-slim AS runtime
WORKDIR /usr/src/shook
# TODO don't do this here
RUN apt-get update && apt-get install libssl1.1 libc6 ca-certificates openssl -y
COPY --from=builder /usr/src/shook/target/release/shook /usr/local/bin
ENTRYPOINT ["/usr/local/bin/shook"]

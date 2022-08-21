FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /usr/src/shook_what_song

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/shook_what_song/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin shook_what_song

FROM debian:bullseye-slim AS runtime
WORKDIR /usr/src/shook_what_song
# TODO don't do this here
RUN apt-get update && apt-get install libssl1.1
COPY --from=builder /usr/src/shook_what_song/target/release/shook_what_song /usr/local/bin
ENTRYPOINT ["/usr/local/bin/shook_what_song"]

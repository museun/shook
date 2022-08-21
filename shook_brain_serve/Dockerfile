FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /usr/src/shook_brain_serve

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/shook_brain_serve/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin shook_brain_serve

FROM debian:bullseye-slim AS runtime
WORKDIR /usr/src/shook_brain_serve
COPY --from=builder /usr/src/shook_brain_serve/target/release/shook_brain_serve /usr/local/bin
ENTRYPOINT ["/usr/local/bin/shook_brain_serve"]

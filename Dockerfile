FROM rust:1.79 AS build

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

# Rust has issues with Alpine at the moment due to its requirement on glibc, so unfortunately we do need to use a slightly larger distro.
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates

# copy the build artifact from the build stage
COPY --from=build ./target/release/google-chats-pr-announcer /

# set the startup command to run your binary
ENTRYPOINT ["/google-chats-pr-announcer"]
FROM rust:1.58 as build

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

# Rust has issues with Alpine at the moment due to its requirement on glibc, so unfortunately we do need to use a slightly larger distro.
FROM debian:bullseye-slim

# copy the build artifact from the build stage
COPY --from=build ./target/release/google-chats-pr-announcer .

# set the startup command to run your binary
CMD ["./google-chats-pr-announcer"]
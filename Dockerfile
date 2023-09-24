FROM rust:1-slim-buster AS build

RUN cargo new --bin rinha
WORKDIR /rinha

COPY Cargo.toml /rinha/
COPY Cargo.lock /rinha/
RUN cargo build --release

COPY src /rinha/src
RUN touch /rinha/src/main.rs
RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /rinha/target/release/rinha-compiladores /rinha

CMD ["/rinha", "/var/rinha/source.rinha.json"]

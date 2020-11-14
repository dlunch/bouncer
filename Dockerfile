#syntax=docker/dockerfile:experimental

FROM rust:stretch as builder
RUN rustup component add rustfmt

WORKDIR /src

COPY . .
RUN --mount=type=cache,target=/src/target cargo install --path . --locked --bins --root build

FROM debian:stretch-slim
COPY --from=builder /src/build/bin /bouncer

EXPOSE 6667

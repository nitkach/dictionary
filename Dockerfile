FROM rust:1.84.1-bullseye as build

WORKDIR /app

COPY . .

ARG SQLX_OFFLINE=true

RUN mkdir bin

RUN --mount=type=cache,id=rust-build,target=/app/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=registry,target=/usr/local/cargo/registry \
    cargo build --target-dir /app/target \
    && cp /app/target/debug/dictionary /app/bin

FROM debian:bullseye

WORKDIR /app

COPY --from=build /app/bin/dictionary /app/bin/dictionary

CMD [ "/app/bin/dictionary" ]

FROM rust:1-alpine3.16

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY ./ /app

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/telegram-api-server

FROM gcr.io/distroless/static-debian11

COPY --from=0 /app/target/x86_64-unknown-linux-musl/release/telegram-api-server .

ENTRYPOINT ["/telegram-api-server"]
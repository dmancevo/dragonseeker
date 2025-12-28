FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY app/Cargo.toml app/Cargo.lock ./
COPY app/src ./src
COPY app/templates ./templates

RUN cargo build --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/dragonseeker .
COPY app/templates ./templates
COPY app/static ./static

EXPOSE 8000

CMD ["./dragonseeker"]

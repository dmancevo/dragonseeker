# Stage 1: Build the Rust binary
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY app/Cargo.toml app/Cargo.lock ./

# Copy source code
COPY app/src ./src

# Build for release
RUN cargo build --release

# Stage 2: Runtime image
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/dragonseeker .

# Copy templates and static files
COPY app/templates ./templates
COPY app/static ./static

EXPOSE 8000

CMD ["./dragonseeker"]

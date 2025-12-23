# Build stage - using nightly for edition 2024 support
FROM rustlang/rust:nightly-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to invalidate the dummy build
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/altenia-backend /app/altenia-backend

# Copy migrations (for reference, though they run via init scripts)
COPY migrations ./migrations

# Expose port
EXPOSE 3000

# Run the binary
CMD ["./altenia-backend"]

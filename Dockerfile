# Build stage
FROM rust:1.82-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# 1) 의존성만 먼저 빌드 (캐싱)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release --no-default-features
RUN rm -rf src

# 2) 실제 소스 복사 후 빌드 (의존성은 캐시 히트)
COPY src/ ./src/
COPY migrations/ ./migrations/
RUN touch src/main.rs
RUN cargo build --release --no-default-features

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates \
    && addgroup -g 1000 appuser \
    && adduser -D -u 1000 -G appuser appuser

WORKDIR /app
COPY --from=builder /app/target/release/rust-backend-base .
COPY migrations/ ./migrations/
RUN chown -R appuser:appuser /app

USER appuser

ENV RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/v1/health || exit 1

CMD ["./rust-backend-base"]

# ------------------------------------------------------------------------------
# 构建
# ------------------------------------------------------------------------------

FROM rust AS builder

ARG FEATURES=""

WORKDIR /app

COPY . .

RUN cargo install --path . --features "$FEATURES" --root /app/install

# ------------------------------------------------------------------------------
# 打包
# ------------------------------------------------------------------------------

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends tzdata && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/config ./config
COPY --from=builder /app/sqlite ./sqlite
COPY --from=builder /app/themes ./themes
COPY --from=builder /app/install/bin/blog-rs /usr/local/bin/blog-rs

RUN chmod +x /usr/local/bin/blog-rs

CMD ["blog-rs"]

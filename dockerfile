# 构建阶段
FROM rust:1.70-slim as builder

# 安装编译依赖
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# 构建（使用 --bin 指定名称）
RUN cargo build --release --bin axum-redis-server

# 运行时阶段
FROM debian:12-slim
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 从构建阶段拷贝二进制
COPY --from=builder /app/target/release/axum-redis-server /usr/local/bin/

CMD ["axum-redis-server"]
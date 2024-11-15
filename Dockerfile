FROM rust:1.80 as builder
WORKDIR /app
COPY . .
ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y libssl3 && \
    rm -rf /var/lib/apt/lists/*
RUN mkdir -p /app

COPY --from=builder /app/target/release/webhook-git-updater /app/

WORKDIR /app
EXPOSE 8080
CMD ["/app/webhook-git-updater"]
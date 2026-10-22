# 線上公開伺服器（Railway）。設定在專案根目錄的 railway.toml；建置時以專案根目錄為 context。
FROM rust:1.89-slim AS build
WORKDIR /app
COPY crates/mahjong-core crates/mahjong-core
COPY crates/mahjong-server crates/mahjong-server
RUN cargo build --release --locked --manifest-path crates/mahjong-server/Cargo.toml

FROM debian:bookworm-slim
COPY --from=build /app/crates/mahjong-server/target/release/mahjong-server /usr/local/bin/mahjong-server
ENV PORT=8080
EXPOSE 8080
CMD ["mahjong-server", "--public"]

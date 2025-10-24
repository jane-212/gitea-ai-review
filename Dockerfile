FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:latest

WORKDIR /app

COPY --from=builder /app/target/release/gitea-ai-review /app/app

EXPOSE 6651

CMD ["/app/app"]

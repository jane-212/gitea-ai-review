FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build

FROM debian:latest

WORKDIR /app

RUN apt-get update \
  && apt-get install -y ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/debug/gitea-ai-review /app/app

EXPOSE 6651

CMD ["/app/app"]

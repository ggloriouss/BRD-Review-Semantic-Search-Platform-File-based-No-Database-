FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# FROM debian:bookworm-slim
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
EXPOSE 8000
CMD ["backend"]
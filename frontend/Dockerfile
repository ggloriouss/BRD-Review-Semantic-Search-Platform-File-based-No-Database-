FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin frontend

FROM nginx:alpine
COPY --from=builder /app/target/release/frontend /usr/share/nginx/html
EXPOSE 3000
CMD ["nginx", "-g", "daemon off;"]
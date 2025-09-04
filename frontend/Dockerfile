FROM rust:latest AS builder
WORKDIR /app

# Install wasm target
RUN rustup target add wasm32-unknown-unknown

# Install trunk (for building Leptos frontend)
RUN cargo install trunk

# Copy sources
COPY . .

# Build frontend as WASM static files
RUN trunk build --release --public-url /

FROM nginx:alpine AS runtime
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
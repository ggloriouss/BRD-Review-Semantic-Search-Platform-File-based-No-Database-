FROM rust:latest AS builder
WORKDIR /app

# install trunk and wasm target
RUN rustup target add wasm32-unknown-unknown \
 && cargo install trunk

COPY . .

# build static assets
RUN trunk build --release --public-url /

FROM node:18-alpine AS runtime
WORKDIR /app

# install a small static server (no SPA fallback)
RUN npm install -g serve

# copy built static files
COPY --from=builder /app/dist ./dist

EXPOSE 8080
# serve WITHOUT `-s` to avoid returning index.html for missing files
CMD ["serve", "dist", "-l", "8080"]
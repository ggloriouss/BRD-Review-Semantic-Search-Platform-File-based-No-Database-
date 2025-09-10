# syntax=docker/dockerfile:1.7
# --- frontend/Dockerfile ---
FROM rust:1.86-bookworm

WORKDIR /app

# ของที่ต้องใช้
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates tar pkg-config libssl-dev xz-utils \
 && rm -rf /var/lib/apt/lists/*

# ให้ rustup เสถียรขึ้นหน่อย
ENV RUSTUP_MAX_RETRIES=10

# ตรวจเวอร์ชัน
RUN rustc -V && cargo -V && rustup -V

# ติดตั้ง toolchain + wasm target ด้วย retry (กันเน็ตสะดุด)
RUN set -eux; \
    rustup set profile minimal; \
    for i in 1 2 3 4 5; do \
      rustup toolchain install stable --profile minimal && break || sleep 5; \
    done; \
    rustup default stable; \
    for i in 1 2 3 4 5; do \
      rustup target add wasm32-unknown-unknown --toolchain stable && break || sleep 5; \
    done; \
    rustup show

# ติดตั้ง Trunk (prebuilt) พร้อม retry ตามสถาปัตยกรรม
ARG TRUNK_VERSION=0.21.14
RUN set -eux; \
    arch="$(uname -m)"; \
    case "$arch" in \
      x86_64)  ASSET="trunk-x86_64-unknown-linux-musl.tar.gz" ;; \
      aarch64) ASSET="trunk-aarch64-unknown-linux-gnu.tar.gz" ;; \
      *) echo "Unsupported arch: $arch"; exit 1 ;; \
    esac; \
    for i in 1 2 3 4 5; do \
      curl -fsSL -o /tmp/trunk.tar.gz "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/${ASSET}" && break || sleep 5; \
    done; \
    tar -C /usr/local/bin -xzf /tmp/trunk.tar.gz trunk; \
    chmod +x /usr/local/bin/trunk; \
    trunk --version

# อุ่น cache ของ dependency (ไม่รัน build จริง)
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    sh -c 'mkdir -p src && echo "fn main() {}" > src/main.rs && cargo fetch'

# คัดลอกโค้ดจริง
COPY . .

# env สำหรับแอป (compose ของคุณตั้ง BACKEND_URL= http://backend:8000 ก็ OK)
ENV RUST_LOG=info

# เปิดพอร์ต dev server
EXPOSE 8080

# รัน Trunk dev server (hot reload)
CMD ["trunk", "serve", "--address", "0.0.0.0", "--port", "8080", "--release"]

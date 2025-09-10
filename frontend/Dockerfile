# --- frontend/Dockerfile ---
FROM rust:1.80-bookworm

WORKDIR /app

# tools พื้นฐาน
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates tar pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

# เพิ่ม WASM target
RUN rustup target add wasm32-unknown-unknown

# ติดตั้ง Trunk แบบ prebuilt (เลือก asset ตามสถาปัตยกรรม)
ARG TRUNK_VERSION=0.21.14
RUN set -eux; \
    arch="$(uname -m)"; \
    case "$arch" in \
      x86_64) ASSET="trunk-x86_64-unknown-linux-musl.tar.gz" ;; \
      aarch64) ASSET="trunk-aarch64-unknown-linux-gnu.tar.gz" ;; \
      *) echo "Unsupported arch: $arch"; exit 1 ;; \
    esac; \
    curl -fL -o /tmp/trunk.tar.gz \
      "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/${ASSET}"; \
    tar -C /usr/local/bin -xzf /tmp/trunk.tar.gz trunk; \
    chmod +x /usr/local/bin/trunk; \
    trunk --version

# อุ่น cache ของ cargo ให้บิลด์เร็วขึ้นในรอบถัดไป
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo fetch

# คอมไพล์แบบลด parallelism เพื่อลดแรม
ENV CARGO_BUILD_JOBS=2

# คัดลอกโค้ดจริง
COPY . .

# เปิดพอร์ต dev server
EXPOSE 8080

# ดีฟอลต์ให้ serve (hot reload) — ใช้กับ docker-compose ของคุณได้เลย
CMD ["trunk", "serve", "--address", "0.0.0.0", "--port", "8080", "--release"]

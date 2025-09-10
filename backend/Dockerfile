FROM rust:1.86-bookworm AS builder
WORKDIR /app

ARG MAKE_JOBS=1
ENV CMAKE_BUILD_PARALLEL_LEVEL=${MAKE_JOBS}
ENV CARGO_BUILD_JOBS=${MAKE_JOBS}

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake git pkg-config clang \
    libjemalloc-dev libsnappy-dev libgflags-dev libtbb-dev libisal-dev \
    libssl-dev zlib1g-dev ca-certificates \
    libboost-all-dev swig python3-dev \
 && rm -rf /var/lib/apt/lists/*

# แสดงเวอร์ชันให้ชัวร์ว่าเป็น toolchain ใหม่
RUN rustc -V && cargo -V

# ---------- เตรียม manifest + build.rs และทำ "prefetch" dependencies ----------
# 1) ต้อง COPY build.rs เข้ามากับ Cargo.toml/Cargo.lock
COPY Cargo.toml Cargo.lock build.rs ./

# 2) สร้าง target หลอก ๆ (จำเป็นต่อ cargo fetch เพราะ manifest ยังไม่มี target ใด ๆ)
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# 3) อุ่นแคช dependency โดยไม่รัน build.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo fetch --locked

# ---------- คัดลอกโค้ดจริงทั้งหมด (ทับ dummy main.rs ไปเลย) ----------
COPY . .

# ---------- สร้าง/ติดตั้ง rocksdb (static) ----------
RUN git clone --depth=1 --filter=blob:none https://github.com/PtilopsisL/rocksdb.git /tmp/rocksdb \
 && cmake -S /tmp/rocksdb -B /tmp/rocksdb/build \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_C_FLAGS="-fPIC" \
      -DCMAKE_CXX_FLAGS="-fPIC -Wno-maybe-uninitialized -Wno-error" \
      -DROCKSDB_BUILD_SHARED=OFF -DROCKSDB_BUILD_STATIC=ON \
      -DWITH_TESTS=OFF -DWITH_TOOLS=OFF -DWITH_BENCHMARK_TOOLS=OFF \
      -DWITH_SNAPPY=ON -DWITH_JEMALLOC=ON -DWITH_GFLAGS=ON -DPORTABLE=1 \
      -DFAIL_ON_WARNINGS=OFF -DDISABLE_WARNING_AS_ERROR=ON \
 && cmake --build /tmp/rocksdb/build --target install -j ${MAKE_JOBS}

# ---------- เตรียม/แพตช์ SPFresh ให้ build แบบ static ----------
RUN if [ ! -d "third_party/SPFresh" ]; then \
      git clone --depth 1 --recurse-submodules https://github.com/SPFresh/SPFresh.git third_party/SPFresh ; \
    fi \
 && if [ ! -d "third_party/SPFresh/ThirdParty/zstd/build/cmake" ]; then \
      rm -rf third_party/SPFresh/ThirdParty/zstd && \
      git clone --depth 1 https://github.com/facebook/zstd.git third_party/SPFresh/ThirdParty/zstd ; \
    fi \
 && sed -i -E 's/^[[:space:]]*add_subdirectory[[:space:]]*\(GPUSupport\).*/# disabled/' third_party/SPFresh/CMakeLists.txt \
 && sed -i -E 's/^[[:space:]]*add_subdirectory[[:space:]]*\(Wrappers\).*/# disabled/' third_party/SPFresh/CMakeLists.txt \
 && sed -i -E 's/^[[:space:]]*add_subdirectory[[:space:]]*\(Test\).*/# disabled/' third_party/SPFresh/CMakeLists.txt \
 && rm -f third_party/SPFresh/AnnService/src/Core/SPANN/ExtraSPDKController.cpp \
 && mkdir -p third_party/SPFresh/AnnService/inc/Core/SPANN \
 && printf '%s\n' \
    '#pragma once' \
    'namespace SPTAG { namespace SPANN {' \
    'struct ExtraSPDKController {' \
    '    static bool Enabled() { return false; }' \
    '    ExtraSPDKController() = default;' \
    '    ~ExtraSPDKController() = default;' \
    '    bool Init() { return false; }' \
    '    void Close() {}' \
    '};' \
    '}}' \
    > third_party/SPFresh/AnnService/inc/Core/SPANN/ExtraSPDKController.h \
 && sed -i -E 's|db\.reset\(\s*new[[:space:]]+SPDKIO\([^;]*\)\s*\);|db.reset();|g' \
    third_party/SPFresh/AnnService/inc/Core/SPANN/ExtraDynamicSearcher.h \
 && sed -i -E '/[Ss][Pp][Dd][Kk]/d' third_party/SPFresh/AnnService/CMakeLists.txt \
 && sed -i -E '/[Ss][Pp][Dd][Kk]/d' third_party/SPFresh/CMakeLists.txt

# ---------- สร้าง SPFresh (static) ----------
RUN cmake -S third_party/SPFresh -B third_party/SPFresh/build \
      -DCMAKE_BUILD_TYPE=Release \
      -DBUILD_SHARED_LIBS=OFF \
      -DGPU=OFF -DLIBRARYONLY=ON -DROCKSDB=ON \
      -DBoost_USE_STATIC_LIBS=ON \
      -DCMAKE_DISABLE_FIND_PACKAGE_CUDA=ON \
      -DCMAKE_DISABLE_FIND_PACKAGE_MPI=ON \
      -DCMAKE_DISABLE_FIND_PACKAGE_uring=ON \
      -DCMAKE_DISABLE_FIND_PACKAGE_SPDK=ON \
      -DZSTD_BUILD_PROGRAMS=OFF -DZSTD_BUILD_SHARED=OFF -DZSTD_BUILD_TESTS=OFF \
      -Wno-dev

# พยายาม build target ที่เป็นไปได้ (บาง fork ใช้ชื่อ SPTAGLib, บางอัน SPTAGLibStatic)
RUN cmake --build third_party/SPFresh/build --target DistanceUtils -j ${MAKE_JOBS} || true && \
    cmake --build third_party/SPFresh/build --target SPTAGLibStatic -j ${MAKE_JOBS} || true && \
    cmake --build third_party/SPFresh/build --target SPTAGLib -j ${MAKE_JOBS} || true

# เก็บ .a ไว้ที่ /usr/local/lib (ค้นหาทั้งหมดใน build/**)
RUN mkdir -p /usr/local/lib /usr/local/include/SPTAG && \
    find third_party/SPFresh/build -type f -name 'lib*.a' -exec cp -a {} /usr/local/lib/ \; && \
    cp -a third_party/SPFresh/AnnService/inc/* /usr/local/include/SPTAG/ || true


# ---------- คอมไพล์ Rust (ตอนนี้ build.rs + headers + libs พร้อม) ----------
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked

########## Runtime ##########
FROM mirror.gcr.io/debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libstdc++6 libjemalloc2 libgomp1 \
    libsnappy1v5 libgflags2.2 libtbb12 libisal2 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

RUN mkdir -p /data

ENV DATA_DIR=/data \
    INDEX_FILE=/data/reviews.index \
    METADATA_FILE=/data/reviews.jsonl \
    MAP_FILE=/data/vector_map.jsonl \
    EMBED_DIM=384 \
    SPFRESH_PARAMS=PostingPageLimit=12 \
    RUST_LOG=info

EXPOSE 8000
CMD ["backend"]

BRD: Review Semantic Search Platform (File-based, No Database)
External Repositories & Resources
SPFresh (vector index)
fastembed-rs (Rust embedding)
axum (Rust web framework)
Dataset: Kaggle - Reviews
1. Objective
สร้างระบบค้นหารีวิวแบบ Semantic ด้วย Vector Search
ข้อมูลและ vector index ทั้งหมด เก็บเป็นไฟล์ (ไม่ใช้ database)
รองรับ insert/append review ได้ไม่จำกัด (ทุกครั้ง append vector & metadata)
ทุกการกระทำ (insert/search) trigger ผ่าน frontend (Leptos) → backend (Rust/axum + fastembed-rs + spfresh)
2. Tech Stack
Backend: Rust (axum), fastembed-rs, spfresh (C++ binding)
Frontend: Leptos (Rust WASM/SSR)
Data store: ไฟล์ index (spfresh) + ไฟล์ metadata (.jsonl)
3. Concise Workflow & BRD Update
3.1 Data Append Flow
ผู้ใช้สามารถ insert ข้อมูลรีวิวใหม่ ได้เรื่อยๆ ผ่านหน้าเว็บ (Frontend Leptos)

ทุกครั้งที่ insert (form หรือ upload):

Frontend ส่งข้อมูลไป Backend
Backend สร้าง embedding vector จาก field ที่กำหนด (เช่น review_body, review_title) ด้วย fastembed-rs
Vector กับข้อมูลรีวิว จะถูก append เข้า spfresh (vector store, ไฟล์) และ metadata (เช่น JSONL ไฟล์)
ไม่ลบ/overwrite — ทุก insert เป็นการ append ข้อมูลใหม่

ข้อมูล index โตต่อเนื่อง (append-only)

3.2 Search Flow
Frontend (หน้า Search) ให้ผู้ใช้กรอกข้อความค้นหา

Frontend ส่ง query ไป backend
Backend สร้าง embedding จาก query → ค้นหาด้วย spfresh (vector store)
คืนผลลัพธ์เป็น list ของ reviews ที่คล้ายคลึง (semantic match) โดย mapping vector id กับ metadata ที่เก็บไว้ในไฟล์
3.3 API Call ทั้งหมดผ่าน Frontend
ทุกการ insert/append (manual หรือ upload) trigger ผ่านหน้าเว็บ
Embedding ทั้งหมด (review & query) ทำฝั่ง backend
Frontend แค่ส่ง raw data/query ไม่ทำ embedding เอง
3.4 Key Points
ไม่มี database ข้อมูลเก็บเป็นไฟล์ vector index (spfresh) + metadata (.jsonl หรือ format อื่น)
ผู้ใช้เพิ่มรีวิวใหม่ได้ไม่จำกัด ข้อมูลจะถูก append ต่อท้าย index store
ไม่มีการลบ/overwrite
Embedding vector สร้างใหม่ทุกครั้งที่ insert
ระบบ frontend เป็นตัวกลางให้ user upload/insert/trigger ทุกอย่าง
3.5 Example Usecase (User Journey)
ผู้ใช้เข้าหน้า Index → กรอกรีวิว/เลือกไฟล์ → กด Upload
Frontend เรียก API (/reviews หรือ /reviews/bulk)
Backend ทำ embedding & append vector/metadata (เขียนลงไฟล์)
ผู้ใช้เข้าหน้า Search → กรอก query → ดูผลลัพธ์รีวิวที่คล้ายที่สุด (semantic search)
4. Project Structure (ASCII Diagram)
project-root/
│
├── frontend/                  # Leptos (Rust SPA/SSR)
│   ├── src/
│   ├── Cargo.toml
│   └── ...
│
├── backend/                   # Rust (axum) + fastembed-rs + spfresh binding
│   ├── src/
│   ├── Cargo.toml
│   ├── spfresh/               # (binding หรือ submodule)
│   ├── data/                  # ← เก็บไฟล์ข้อมูล (append-only)
│   │   ├── reviews.index      # ← spfresh vector index (binary, append-only)
│   │   └── reviews.jsonl      # ← metadata: 1 review ต่อ 1 บรรทัด (JSON Lines)
│   └── ...
│
├── docker-compose.yml         # Orchestration ทั้งระบบ (no DB)
│
└── README.md
5. Data Storage Concept
Vector Index: เก็บที่ data/reviews.index ด้วย spfresh (append-only)

Metadata: เก็บที่ data/reviews.jsonl (JSON lines) mapping กับ vector index ตามลำดับ

ตัวอย่าง 1 บรรทัดใน reviews.jsonl:

{"review_title": "Great phone", "review_body": "Battery lasts long", "product_id": "P123", "review_rating": 5}
ไม่มี database ทุกอย่างเป็นไฟล์

6. ขยายงาน/Implement เพิ่มเติม
เพิ่ม field/validate schema: เปลี่ยนได้ที่ฝั่ง frontend + backend
โครงสร้างไฟล์ (เช่น data/reviews.index, data/reviews.jsonl) กำหนดเองได้
สามารถ build docker-compose, deploy, production-ready ได้ตามที่สรุปก่อนหน้า
6.1 โครงสร้างไฟล์โปรเจกต์ (ASCII Diagram)
project-root/
│
├── frontend/                  # Leptos (Rust SPA/SSR)
│   ├── src/
│   ├── Cargo.toml
│   └── ...
│
├── backend/                   # Rust (axum) + fastembed-rs + spfresh binding
│   ├── src/
│   ├── Cargo.toml
│   ├── spfresh/               # (binding หรือ submodule)
│   ├── data/                  # ← เก็บไฟล์ข้อมูล (append-only)
│   │   ├── reviews.index      # ← spfresh vector index (binary, append-only)
│   │   └── reviews.jsonl      # ← metadata: 1 review ต่อ 1 บรรทัด (JSON Lines)
│   └── ...
│
├── docker-compose.yml         # Orchestration ทั้งระบบ (no DB)
│
└── README.md
6.2 ตัวอย่าง Flow ของการ insert & search
(A) Insert/Append Review
[Frontend (Leptos)]
    |
    |  POST /reviews (JSON หรือ bulk)
    V
[Backend (axum + fastembed-rs)]
    |
    |---> สร้าง embedding vector
    |---> append vector → [data/reviews.index]
    |---> append metadata → [data/reviews.jsonl]
(B) Semantic Search
[Frontend (Leptos)]
    |
    |  POST /search (query)
    V
[Backend (axum + fastembed-rs)]
    |
    |---> สร้าง embedding vector จาก query
    |---> ค้นหาใกล้เคียงใน [data/reviews.index] ด้วย spfresh
    |---> mapping vector id → บรรทัดที่ตรงใน [data/reviews.jsonl]
    |---> return review (JSON) list to frontend
6.3 หมายเหตุ
ถ้า format/field metadata เปลี่ยน ให้ update ทั้งฝั่ง frontend และ backend
สามารถสำรอง/ย้ายไฟล์ reviews.index กับ reviews.jsonl ได้ง่าย
ไม่มี database ทุกอย่างเป็น append-only file
scale ได้ง่าย, backup/restore ง่าย, deploy dev/prod ไม่ซับซ้อน
7. docker-compose.yml (No DB)
version: "3.9"
services:
  backend:
    build: ./backend
    container_name: rust-backend
    ports:
      - "8000:8000"
    volumes:
      - ./backend/data:/app/data   # backend เก็บไฟล์ index/metadata ลง host

  frontend:
    build: ./frontend
    container_name: leptos-frontend
    ports:
      - "3000:3000"
    volumes:
      - ./frontend:/app
    environment:
      - BACKEND_URL=http://backend:8000
8. Key Points
Insert/append-only, ข้อมูลใหม่จะเพิ่มไปเรื่อยๆ (ไม่มีการลบ/แก้ไข)
ไม่มี database ใดๆ ทั้งสิ้น
Mapping vector id <-> metadata line ทำจากลำดับ index (0-based)
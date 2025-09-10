# BRD: Review Semantic Search Platform (File-based, No Database)

A file-based semantic review search system with a Rust backend and Leptos frontend. Reviews are embedded locally and indexed for fast semantic search—no external database required.

---

## Features

- **Insert Reviews:** Add reviews via web UI (single or bulk upload)
- **Semantic Search:** Find similar reviews using natural language queries
- **Local Embeddings:** Uses fastembed-rs for text embedding (no network calls)
- **Append-Only Storage:** Durable, crash-safe file-backed persistence
- **Vector Search:** High-performance ANN search with SPFresh/SPTAG
- **Dockerized:** Easy deployment with Docker Compose

---

## Architecture

- **Frontend:** Leptos (Rust WASM)
- **Backend:** Rust (axum)
- **Embeddings:** fastembed-rs
- **Vector Index:** SPFresh/SPTAG (native, via FFI)
- **Storage:** JSON Lines & binary vector index files

---

## Data Flow

- **Insert:**  
  Frontend form → POST `/reviews` → backend → embed → append to index & metadata
- **Search:**  
  Frontend query → POST `/search` → backend → embed → vector search → return matches

---

## Quick Start

### Prerequisites

- Docker & Docker Compose

### Steps

1. **Clone the repository**
    ```
    git clone https://github.com/ggloriouss/BRD-Review-Semantic-Search-Platform-File-based-No-Database-.git
    cd BRD-Review-Semantic-Search-Platform-File-based-No-Database-
    ```

2. **Run with Docker Compose**
    ```
    docker-compose up --build
    ```

3. **Access the app**
    - Frontend: [http://localhost:80](http://localhost:80)
    - Backend API: [http://localhost:8000](http://localhost:8000)

---

## Usage

### Insert Reviews

- Go to the frontend URL.
- Fill in review details (review text, rating).
- Click "Upload" to submit.
- For bulk upload, paste a JSON array:
    ```json
    [
      {"review": "Great food, will come again!", "rating": 5},
      {"review": "Service was slow.", "rating": 2}
    ]
    ```

### Search Reviews

- Enter a search query in natural language.
- View semantically ranked results.

---

## Project Structure

```
project-root/
├── backend/                         # Rust backend service
│   ├── data/                        # Data storage directory
│   │   ├── reviews.index            # SPFresh vector index (binary)
│   │   ├── reviews.jsonl            # Metadata (JSON Lines format)
│   │   ├── vector_map.jsonl         # Vector ID to review ID mapping
│   │   ├── TestReviews.csv          # Example CSV for bulk insert
│   ├── native/                      # Native C++ FFI bindings for SPFresh
│   │   ├── spfresh_c_api.cc
│   │   ├── spfresh_c_api.h
│   ├── src/                         # Rust source code
│   │   ├── embedder.rs              # Text embedding generation
│   │   ├── handlers.rs              # API endpoint handlers
│   │   ├── main.rs                  # Application entry point
│   │   ├── routes.rs                # API route definitions
│   │   ├── spfresh.rs               # FFI wrapper for SPFresh
│   │   ├── storage.rs               # File I/O operations
│   │   └── types.rs                 # Data structure definitions
│   ├── third_party/
│   │   └── SPFresh/                 # SPFresh/SPTAG vector engine source
│   ├── Cargo.toml                   # Rust dependencies
│   ├── Dockerfile                   # Backend Dockerfile
├── frontend/                        # Leptos frontend
│   ├── src/
│   │   ├── api.rs                   # Backend API client
│   │   ├── main.rs                  # Frontend entry point
│   │   └── components/              # UI components
│   │       ├── insert_review.rs     # Review insertion form
│   │       ├── mod.rs               # Component exports
│   │       └── search.rs            # Search interface
│   ├── index.html                   # HTML template
│   ├── Cargo.toml                   # Frontend Rust dependencies
│   ├── Dockerfile                   # Frontend Dockerfile
│   ├── Trunk.toml                   # Trunk configuration
├── docker-compose.yml               # Docker Compose configuration
└── README.md                        # Project documentation
```

---

## API Endpoints

### `POST /reviews` — Insert a single review

Request:
```json
{
  "review": "Great food, will come again!",
  "rating": 5
}
```
Response:
```json
{
  "id": "uuid",
  "review": "Great food, will come again!",
  "rating": 5,
  "schema_version": "v1",
  "vector_id": 0
}
```

### `POST /reviews/bulk` — Insert multiple reviews

Request:
```json
[
  {"review": "Great food, will come again!", "rating": 5},
  {"review": "Service was slow.", "rating": 2}
]
```
Response:
```json
[
  {
    "id": "uuid1",
    "review": "Great food, will come again!",
    "rating": 5,
    "schema_version": "v1",
    "vector_id": 0
  },
  {
    "id": "uuid2",
    "review": "Service was slow.",
    "rating": 2,
    "schema_version": "v1",
    "vector_id": 1
  }
]
```

### `POST /search` — Semantic search

Request:
```json
{
  "query": "martini",
  "top_k": 10
}
```
Response:
```json
{
  "hits": [
    {
      "review": {
        "id": "uuid2",
        "review": "Olive or Twist is the historic site of my VERY FIRST MARTINI...",
        "rating": 1,
        "schema_version": "v1",
        "vector_id": 1
      },
      "score": 0.98
    }
  ]
}
```

---

## Development

### Local Development

1. **Run backend**
    ```
    cd backend
    cargo run
    ```

2. **Run frontend**
    ```
    cd frontend
    trunk serve
    ```

3. **Build Docker images**
    ```
    docker-compose build
    ```

---

## Notes

- All data is append-only for durability.
- Embeddings are generated locally; no external API calls.
- Vector index and metadata are mapped by line number.

---

## License

See [LICENSE](LICENSE) for details.

---

Created as part of the BRD Review Semantic Search Platform project. Uses file-based storage with
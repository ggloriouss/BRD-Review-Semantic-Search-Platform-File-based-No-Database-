# BRD: Review Semantic Search Platform (File-based, No Database)

A file-backed semantic review search system that ingests unlimited user reviews via a Leptos frontend, embeds them using fastembed-rs, and performs approximate nearest neighbor (ANN) vector search with SPFresh/SPTAG index—no external database required. All data lives in files under `backend/data`.

## Features

1. **Insert Reviews:** Add new reviews continuously through a web interface (manual entry or bulk upload)
2. **Semantic Search:** Find the most semantically similar reviews to any search query
3. **Local Embeddings:** Generate vector embeddings with fastembed-rs (no network dependencies)
4. **Append-Only Storage:** Crash-safe, file-backed persistence with no database requirements
5. **Vector Search:** Fast ANN search using SPFresh/SPTAG for high-performance semantic matching
6. **Dockerized:** Simple deployment with docker-compose

## Technical Architecture

**Stack**
- Frontend: Leptos (Rust WASM framework for interactive UI)
- Backend: Rust with axum web framework
- Embeddings: fastembed-rs for local text embedding generation
- Vector Index: SPFresh/SPTAG for approximate nearest neighbor search
- Storage: File-based (JSON Lines + binary vector index)

**Data Flow**
- **Insert Flow:**  
  Frontend form → POST `/reviews` → backend handler  
  → Generate embedding → append to vector index → append to metadata file

- **Search Flow:**  
  Frontend search → POST `/search` → backend handler  
  → Embed query → vector search → map results to metadata → return ranked matches

## Installation and Setup

### Prerequisites
- Docker and Docker Compose

### Quick Start

1. Clone the repository:
    ```
    git clone https://github.com/ggloriouss/BRD-Review-Semantic-Search-Platform-File-based-No-Database-.git
    cd BRD-Review-Semantic-Search-Platform-File-based-No-Database-
    ```

2. Run with Docker Compose:
    ```
    docker-compose up --build
    ```

3. Access the Application:
    - Frontend: http://localhost:80
    - Backend API: http://localhost:8000

## Usage

### Insert Reviews

- Navigate to http://localhost:80
- Fill in the review details:
    - Review (Required)
    - Rating (Required, integer 0-5)
- Click "Upload" to submit

- For bulk upload, paste a JSON array of objects:
    ```json
    [
      {"review": "Great food, will come again!", "rating": 5},
      {"review": "Service was slow.", "rating": 2}
    ]
    ```

### Search Reviews

- Enter your search query in natural language
- View semantically similar review results ranked by relevance

## Project Structure

```
project-root/
├── backend/                   # Rust backend service
│   ├── data/                  # Data storage directory
│   │   ├── reviews.index      # SPFresh vector index (binary)
│   │   ├── reviews.jsonl      # Metadata (JSON Lines format)
│   │   ├── vector_map.jsonl   # Vector ID to review ID mapping
│   ├── src/                   # Source code
│   │   ├── embedder.rs        # Text embedding generation
│   │   ├── handlers.rs        # API endpoint handlers
│   │   ├── main.rs            # Application entry point
│   │   ├── routes.rs          # API route definitions
│   │   ├── storage.rs         # File I/O operations
│   │   └── types.rs           # Data structure definitions
├── frontend/                  # Leptos frontend
│   ├── src/
│   │   ├── components/        # UI components
│   │   │   ├── insert_review.rs # Review insertion form
│   │   │   ├── mod.rs         # Component exports
│   │   │   └── search.rs      # Search interface
│   │   ├── api.rs             # Backend API client
│   │   └── main.rs            # Frontend entry point
│   ├── index.html             # HTML template
│   └── Trunk.toml             # Trunk configuration
└── docker-compose.yml         # Docker Compose configuration
```

## Implementation Details

### Append-Only Storage

- All data is stored in append-only files to ensure durability
- No deletions or updates are performed, only appends
- File locks prevent race conditions during writes

### Vector Index

- The `reviews.index` file stores embeddings for fast similarity search
- SPFresh/SPTAG provides efficient approximate nearest neighbor search
- Vector IDs map to metadata via sequential line numbers in the JSONL file

### Embedding Generation

- Reviews are embedded using fastembed-rs (no network calls)
- Search queries go through the same embedding process
- Cosine similarity is used to rank search results

## API Endpoints

### 1. `POST /reviews`: Insert a single review

Request:
```json
{
  "review": "I came here before a pirates game, so it was around 5:30ish or so in the evening, ...",
  "rating": 1
}
```
Response:
```json
{
  "id": "uuid",
  "review": "Olive or Twist is the historic site of my VERY FIRST MARTINI when I turned 21, many years ago...",
  "rating": 1,
  "schema_version": "v1",
  "vector_id": 0
}
```

### 2. `POST /reviews/bulk`: Insert multiple reviews

Request:
```json
[
  {
    "review": "I came here before a pirates game, so it was around 5:30ish or so in the evening, ...",
    "rating": 1
  },
  {
    "review": "Olive or Twist is the historic site of my VERY FIRST MARTINI when I turned 21, many years ago...",
    "rating": 1
  }
]
```
Response:
```json
[
  {
    "id": "uuid1",
    "review": "I came here before a pirates game, so it was around 5:30ish or so in the evening, ...",
    "rating": 1,
    "schema_version": "v1",
    "vector_id": 0
  },
  {
    "id": "uuid2",
    "review": "Olive or Twist is the historic site of my VERY FIRST MARTINI when I turned 21, many years ago...",
    "rating": 1,
    "schema_version": "v1",
    "vector_id": 1
  }
]
```

### 3. `POST /search`: Semantic search

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
        "review": "Olive or Twist is the historic site of my VERY FIRST MARTINI when I turned 21, many years ago. ...",
        "rating": 1,
        "schema_version": "v1",
        "vector_id": 1
      },
      "score": 0.98
    },
    {
      "review": {
        "id": "uuid1",
        "review": "I came here before a pirates game, so it was around 5:30ish or so in the evening, ...",
        "rating": 1,
        "schema_version": "v1",
        "vector_id": 0
      },
      "score": 0.85
    }
  ]
}
```

## Development

### Local Development

1. Run the backend:
    ```
    cd backend
    cargo run
    ```

2. Run the frontend:
    ```
    cd frontend
    trunk serve
    ```

3. Building Docker Images
    ```
    docker-compose build
    ```

## Notes on WASM/uuid

If you build the frontend for WASM, make sure your `frontend/Cargo.toml` includes:
```toml
uuid = { version = "1", features = ["v4", "serde", "js"] }
```
This enables randomness for UUID generation in browsers.

## License

This project is licensed under the terms specified in the LICENSE file (root/LICENSE).

---

Created as part of the BRD Review Semantic Search Platform project. Uses file-based storage with no database dependencies.
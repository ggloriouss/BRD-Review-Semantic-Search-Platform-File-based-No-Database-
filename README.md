
# BRD: Review Semantic Search Platform (File-based, No Database)
A file-backed semantic review search system that ingests unlimited user reviews via a Leptos frontend, embeds them using fastembed-rs, and performs approximate nearest neighbor (ANN) vector search with SPFresh/SPTAG index—no external database required. All data lives in files under ``` backend\data ``` (data folder).

## Features
01. Insert Reviews: Add new reviews continuously through a web interface (manual entry or bulk upload)

02. Semantic Search: Find the most semantically similar reviews to any search query

03. Local Embeddings: Generate vector embeddings with fastembed-rs (no network dependencies)

04. Append-Only Storage: Crash-safe, file-backed persistence with no database requirements

05. Vector Search: Fast ANN search using SPFresh/SPTAG for high-performance semantic matching

06. Dockerized: Simple deployment with docker-compose

## Technical Architecture
### Stack
- Frontend: Leptos (Rust WASM framework for interactive UI)

- Backend: Rust with axum web framework

- Embeddings: fastembed-rs for local text embedding generation

- Vector Index: SPFresh/SPTAG for approximate nearest neighbor search

- Storage: File-based (JSON Lines + binary vector index)

### Data Flow
01. Insert Flow:
    - Frontend form → POST /reviews → handlers::insert_review_handler
    - Generate embedding → append to vector index → append to metadata file

02. Search Flow:
    - Frontend search → POST /search → handlers::search_handler
    - Embed query → vector search → map results to metadata → return ranked matches

## Installation and Setup
### Prerequisites
    - Docker and Docker Compose

### Quick Start
01. Clone the repository:

    ``` 
    git clone https://github.com/ggloriouss/BRD-Review-Semantic-Search-Platform-File-based-No-Database-.git
    cd D:\BRD-Review-Semantic-Search-Platform-File-based-No-Database-
    ```
    Please don't forget to check your file directory before 'cd'

02. Run with Docker Compose:
    ``` 
    docker-compose up --build
    ```

03. Access the Application:
    - Frontend: http://localhost:3000
    - Backend API: http://localhost:8000

## Usage
### Insert Reviews
    01. Navigate to http://localhost:3000/insert
    02. Fill in the review details:
        - Title (Optional)
        - Review Body (Required)
        - Rating (Optional)
    03. Click "Upload" to submit

### Search Reviews
    01. Navigate to http://localhost:3000/search
    02. Enter your search query in natural language
    03. View semantically similar review results ranked by relevance

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
- The ``` reviews.index ``` file stores embeddings for fast similarity search
- SPFresh/SPTAG provides efficient approximate nearest neighbor search
- Vector IDs map to metadata via sequential line numbers in the JSONL file

### Embedding Generation
- Reviews are embedded using fastembed-rs (no network calls)
- Search queries go through the same embedding process
- Cosine similarity is used to rank search results

## API Endpoints
01. ```POST /reviews```: Insert a single review
    ``` 
    {
        "title": "Optional title",
        "body": "Review content goes here",
        "rating": 5
    }
    ```

02. ```POST /reviews/bulk```: Insert multiple reviews
    ``` 
    [
        {"title": "Review 1", "body": "Content 1", "rating": 4},
        {"body": "Content 2"}
    ]
    ```

03. ```POST /search```: Semantic search
    ```
    {
        "query": "battery life",
        "top_k": 10
    }
    ```

## Development
### Local Development
01. Run the backend:
```
cd backend
cargo run
```

02. Run the frontend:
```
cd frontend
trunk serve
```

03. Building Docker Images
```
docker-compose build
```

## License
This project is licensed under the terms specified in the LICENSE file (root/LICENSE).

---
Created as part of the BRD Review Semantic Search Platform project. Uses file-based storage with no database dependencies.
=======

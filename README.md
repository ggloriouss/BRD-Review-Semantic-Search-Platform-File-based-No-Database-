# Semantic Review Search Platform

A lightweight, file-based semantic search system for user reviews built with Rust. Leverages Leptos for the frontend, fastembed-rs for embeddings, and SPFresh/SPTAG for vector search - all without external databases.

## Core Features

- **Review Management**: Add reviews via web interface (manual/bulk upload)
- **Semantic Search**: Find similar reviews using cosine similarity
- **File-Based Storage**: Durable, append-only storage without databases
- **Vector Search**: Fast approximate nearest neighbor (ANN) search
- **Containerized**: Easy deployment with Docker Compose

## Technology Stack

- **Backend**: Rust (axum), fastembed-rs, spfresh
- **Frontend**: Leptos (Rust WASM/SSR)  
- **Storage**: File-based (vector index + JSONL)
- **Deploy**: Docker Compose

## Quick Start

1. Prerequisites:
    - Docker and Docker Compose
    - Rust toolchain (for development)

2. Run with Docker:
    ```bash
    docker-compose up --build
    ```
    Access at:
    - Frontend: http://localhost:3000
    - API: http://localhost:8000

3. Local Development:
    ```bash
    # Backend
    cd backend && cargo run
    
    # Frontend 
    cd frontend && cargo run
    ```

## Project Structure
```
project-root/
│
├── frontend/                  # Leptos (Rust SPA/SSR)
│   ├── src/
│   │   ├── api.rs             # API client for backend calls
│   │   ├── main.rs            # App entry point with routing
│   │   └── components/        # UI components (InsertReview, Search)
│   │       ├── mod.rs
│   │       ├── insert_review.rs
│   │       └── search.rs
│   ├── Cargo.toml
│   └── Dockerfile
│
├── backend/                   # Rust (axum) + fastembed-rs + spfresh binding
│   ├── src/
│   │   ├── main.rs            # Server entry point
│   │   ├── handlers.rs        # API handlers (insert, search)
│   │   ├── routes.rs          # Route definitions
│   │   ├── storage.rs         # File I/O for index and metadata
│   │   ├── embedder.rs        # Embedding logic
│   │   └── types.rs           # Data structures
│   ├── Cargo.toml
│   ├── spfresh/               # C++ binding or submodule
│   ├── data/                  # Append-only data files
│   │   ├── reviews.index      # SPFresh vector index (binary)
│   │   └── reviews.jsonl      # Metadata (JSON Lines, 1 review per line)
│   └── Dockerfile
│
├── docker-compose.yml         # Orchestration (backend + frontend)
├── .gitignore
├── Prompt.txt                 # Project requirements and tech stack
├── Instructions.md            # Detailed implementation guide
└── README.md                  # This file
```
## API Endpoints

- `POST /reviews`: Add single review
- `POST /reviews/bulk`: Bulk import reviews
- `POST /search`: Search reviews

## Data Storage

- Append-only files in `data/` directory
- Vector index (.index) maps to metadata (.jsonl)
- Zero external dependencies

## Contributing

1. Use specified tech stack
2. Add tests with `cargo test`
3. Update documentation
4. Submit PR

## License

MIT
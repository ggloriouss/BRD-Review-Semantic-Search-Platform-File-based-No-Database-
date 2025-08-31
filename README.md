# BRD-Review-Semantic-Search-Platform-File-based-No-Database-
Goal Build a file‑backed semantic review search that can ingest unlimited user reviews via a Leptos frontend, embed them with fastembed-rs, and perform ANN vector search using an SPFresh/SPTAG index—no external DB. All data lives in files under backend/data/.

## 1.) Scope & Objectives
O1. Insert reviews via frontend → backend API.

O2. Generate embeddings locally (no network models).

O3. Append‑only persistence: reviews.index (vector index) + reviews.jsonl (metadata).

O4. Top‑K semantic search with cosine similarity.

O5. File‑based durability and crash‑safe (best‑effort) writes.

O6. Dockerized dev run; docker-compose with no databases.

## 2.) Solved
3.1 Insert Flow: InsertReview frontend -> POST /reviews → handlers::insert_review_handler → embed -> SpFreshIndex::append_vector -> append_metadata -> append_vector_map

3.2 Search Flow: Search frontend -> POST /search → handlers::search_handler → embed query -> SpFreshIndex::search -> read_vector_map -> read_metadata_by_review_ids -> return results

3.3 API via Frontend: frontend code calls /reviews, /reviews/bulk, /search — backend routes wired in routes.rs

3.4 Append-only key points: append_metadata and append_vector_map use append-only file writes; locks on index append to avoid race conditions

3.5 User Journey: shown with frontend components and curl-like calls previously
# API-DOCS — Waypoints, Payloads and Services (updated)

This document describes the HTTP API used by the Leptos frontend and the Rust backend, the JSON payloads, and where code for each endpoint lives in the repository. It also lists which frontend/backend functions call each other.

Notes about today's changes
- The system was refactored to store and exchange only two review fields: `review` (String) and `rating` (integer). See sample CSV: [backend/data/TestReviews.csv](backend/data/TestReviews.csv).
- Frontend API calls are proxied under `/api` in development (`frontend/Trunk.toml`), so client helpers post to `/api/*`. See frontend client: [frontend/src/api.rs](frontend/src/api.rs).
- WASM build notes: enable `uuid` feature `js` in [frontend/Cargo.toml](frontend/Cargo.toml) and ensure the wasm target is installed (`rustup target add wasm32-unknown-unknown`). See details below.

---

## Overview (high level)
- Backend: axum HTTP server exposes endpoints for health, insert (single/bulk) and semantic search. Data and vectors are stored in files under `backend/data`.
- Frontend: Leptos UI calls backend endpoints via [frontend/src/api.rs](frontend/src/api.rs).
- Data model: review text + rating are stored and referenced by a generated id and vector_id. Vector index (SPFresh) holds embeddings; metadata stored in JSONL.

---

## Endpoints (waypoints) — updated paths and payloads

All HTTP paths are mounted under `/api` by the frontend dev server/proxy. In production your reverse-proxy may remove `/api`; adjust [frontend/Trunk.toml](frontend/Trunk.toml) or environment accordingly.

1) GET /api/health
- Purpose: simple liveness/health check.
- Request: none
- Response: 200 OK JSON `{ "status": "ok" }`
- Backend handler: [`health_handler`](backend/src/handlers.rs) — [backend/src/handlers.rs](backend/src/handlers.rs)
- Frontend caller: optional health checks in [frontend/src/api.rs](frontend/src/api.rs)

2) POST /api/reviews
- Purpose: insert a single review
- Request JSON (body):
```json
{
  "review": "Full review text string",
  "rating": 1
}
```
- Response JSON (body): StoredReview
```json
{
  "id": "uuid-v4",
  "review": "Full review text string",
  "rating": 1,
  "schema_version": "v1",
  "vector_id": 0
}
```
- Backend handler: [`insert_review_handler`](backend/src/handlers.rs) — [backend/src/handlers.rs](backend/src/handlers.rs)
  - Input type: [`ReviewInput`](backend/src/types.rs) — [backend/src/types.rs](backend/src/types.rs)
  - Steps:
    - Validate input (`review` non-empty, `rating` range)
    - Embed text via [`Embedder::embed_one`](backend/src/embedder.rs) — [backend/src/embedder.rs](backend/src/embedder.rs)
    - Append vector to index via [`SpFreshIndex::append_vector`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
    - Persist metadata via [`append_review_line`](backend/src/storage.rs) and [`append_vector_map_line`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
- Frontend caller: [`create_review`](frontend/src/api.rs) — [frontend/src/api.rs](frontend/src/api.rs)
- Example: one of the records in [backend/data/TestReviews.csv](backend/data/TestReviews.csv) corresponds to the request above.

3) POST /api/reviews/bulk
- Purpose: insert multiple reviews in a single request.
- Request JSON (body): JSON array of ReviewInput objects
```json
[
  { "review": "First review text...", "rating": 1 },
  { "review": "Second review text...", "rating": 1 }
]
```
- Response JSON: array of StoredReview objects (one per input)
- Backend handler: [`bulk_insert_handler`](backend/src/handlers.rs) — [backend/src/handlers.rs](backend/src/handlers.rs)
  - Input type: [`BulkReviews`](backend/src/types.rs) — [backend/src/types.rs](backend/src/types.rs)
  - Batch embedding uses [`Embedder::embed`](backend/src/embedder.rs) for better throughput.
  - Each item appended as in single insert (vector + metadata).
- Frontend caller: [`create_bulk`](frontend/src/api.rs) — [frontend/src/api.rs](frontend/src/api.rs)

4) POST /api/search
- Purpose: semantic search (nearest neighbors)
- Request JSON (body):
```json
{
  "query": "martini",
  "top_k": 10
}
```
- Response JSON:
```json
{
  "hits": [
    {
      "review": { "id":"uuid2", "review":"...","rating":1,"schema_version":"v1","vector_id":1 },
      "score": 0.98
    }
  ]
}
```
- Backend handler: [`search_handler`](backend/src/handlers.rs) — [backend/src/handlers.rs](backend/src/handlers.rs)
  - Input type: [`SearchRequest`](backend/src/types.rs) and output [`SearchResponse`]/[`SearchHit`] — [backend/src/types.rs](backend/src/types.rs)
  - Steps:
    - Embed query via [`Embedder::embed_one`](backend/src/embedder.rs) — [backend/src/embedder.rs](backend/src/embedder.rs)
    - Perform ANN search via [`SpFreshIndex::search`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
    - Map vector_id -> metadata loaded from [`load_all_reviews`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
- Frontend caller: [`search`](frontend/src/api.rs) — [frontend/src/api.rs](frontend/src/api.rs)

---

## Data structures (where defined)
- Backend: [backend/src/types.rs](backend/src/types.rs)
  - [`ReviewInput`](backend/src/types.rs) — payload for /reviews and /reviews/bulk
  - [`StoredReview`](backend/src/types.rs) — persisted metadata returned to clients
  - [`BulkReviews`](backend/src/types.rs) — wrapper for bulk endpoint
  - [`SearchRequest`], [`SearchResponse`], [`SearchHit`] — search API types
- Frontend mirrors types in [frontend/src/api.rs](frontend/src/api.rs): `ReviewInput`, `StoredReview`, `SearchRequest`, `SearchHit`, `SearchResponse`

---

## Storage and index mapping
- Vector index file: `backend/data/reviews.index` (append-only binary). Managed via [`SpFreshIndex`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
- Metadata file: `backend/data/reviews.jsonl` (one JSON object per line) — written by [`append_review_line`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
- Optional vector map file: `backend/data/vector_map.jsonl` (vector_id → review_id) — written by [`append_vector_map_line`](backend/src/storage.rs) — [backend/src/storage.rs](backend/src/storage.rs)
- Mapping rule: vector_id is the index position in `reviews.index` (0-based) and corresponds to the metadata entry for the same insertion order.

---

## Frontend build & dev notes (today's updates)
- Dev proxy: [frontend/Trunk.toml](frontend/Trunk.toml) includes a proxy that rewrites `/api` to the backend during `trunk serve`. See [frontend/Trunk.toml](frontend/Trunk.toml).
- `frontend/src/api.rs` functions now default to the proxied base `/api` via `api_base()` — [frontend/src/api.rs](frontend/src/api.rs).
- WASM UUID: For `uuid::new_v4()` in the browser, enable the `js` feature in [frontend/Cargo.toml](frontend/Cargo.toml):
```toml
uuid = { version = "1", features = ["v4", "serde", "js"] }
```
- Ensure wasm target is installed locally:
```sh
rustup target add wasm32-unknown-unknown
```
- Trunk watch fix: [frontend/Trunk.toml](frontend/Trunk.toml) `[watch].ignore` updated to avoid invalid paths (remove `./src/target`).

---

## Example requests using repository sample data
- Single insert (use one TestReviews.csv entry):
  - Request: JSON with `review` and `rating` as in [backend/data/TestReviews.csv](backend/data/TestReviews.csv)
  - Call: [`create_review`](frontend/src/api.rs) → posts to `/api/reviews`
- Bulk insert:
  - Request: JSON array of objects (example in `frontend/` components)
  - Call: [`create_bulk`](frontend/src/api.rs) → posts to `/api/reviews/bulk`
- Search:
  - Request: `{ "query":"martini", "top_k": 10 }`
  - Call: [`search`](frontend/src/api.rs) → posts to `/api/search`

---

## Troubleshooting & debugging tips (concise)
- MIME / Wasm load errors: ensure `trunk build --public-url /` and static server serves `.wasm` as `application/wasm`.
- `uuid` build error on WASM: enable `js` feature in [frontend/Cargo.toml](frontend/Cargo.toml).
- Missing wasm target: run `rustup target add wasm32-unknown-unknown`.
- Docker build timeouts/pull errors: network connectivity or Docker registry throttling — try `docker pull nginx:alpine` then rebuild. See `docker-compose.yml` in repo root.
- Trunk watch path canonicalization errors: check [frontend/Trunk.toml](frontend/Trunk.toml) and remove incorrect `./src/target`.

---

## File quick-links
- Backend handlers: [backend/src/handlers.rs](backend/src/handlers.rs)
- Backend types: [backend/src/types.rs](backend/src/types.rs)
- Backend embedder: [backend/src/embedder.rs](backend/src/embedder.rs)
- Backend storage / SpFresh index: [backend/src/storage.rs](backend/src/storage.rs)
- Sample CSV: [backend/data/TestReviews.csv](backend/data/TestReviews.csv)
- Frontend API client: [frontend/src/api.rs](frontend/src/api.rs)
- Frontend Trunk config: [frontend/Trunk.toml](frontend/Trunk.toml)
- Frontend Cargo: [frontend/Cargo.toml](frontend/Cargo.toml)
- Orchestration: [docker-compose.yml](docker-compose.yml)

---

If you want, I can:
- Add curl examples for each endpoint using the TestReviews.csv data, or
- Produce a minimal end-to-end example script that posts the CSV entries via the frontend client
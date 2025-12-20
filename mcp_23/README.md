# mcp_23

 Version: 0.9.1

 date    : 2025/12/20

 update :

***

Rust RAG Search , vector data

* embedding: gemini-embedding-001
* Postgres pgvector use

***
* .env
```
POSTGRES_CONNECTION_STR="postgres://root:admin@localhost/mydb"
GEMINI_API_KEY=your-key
```
***
* build

```
cargo build
cargo run
```

***
* data path: ./data

***
* table

```
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE documents (
  id SERIAL PRIMARY KEY,
  content TEXT NOT NULL,
  embedding vector(1024)
);
```
***




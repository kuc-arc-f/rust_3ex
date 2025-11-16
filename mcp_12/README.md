# mcp_12

 Version: 0.9.1

 date    : 2025/11/14

 update :

***

Rust , RAG vector data add

* model: gemini-embedding-001
* Postgres sqlx use

***
### setup

* build

```
cargo build
cargo run
```
***
* data file path: ./data

***
* .env
* POSTGRES_CONNECTION_STR set
* GEMINI_API_KEY set

```
POSTGRES_CONNECTION_STR=postgres://postgres:admin@localhost/postgres
GEMINI_API_KEY=your-key
```

***
* table: table.sql
```
CREATE TABLE IF NOT EXISTS embeddings (
  id TEXT PRIMARY KEY,
  sessid TEXT,
  name TEXT,
  content TEXT,
  embeddings BYTEA
);

```

***


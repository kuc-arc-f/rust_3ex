# mcp_19

 Version: 0.9.1

 date    : 2025/11/26

 update :

***

Rust RAG Search , Ollama pgvector

* model-embed: qwen3-embedding:0.6b
* model: gemini-2.0-flash
* Postgres sqlx use

***
### setup
***
* .env
* POSTGRES_CONNECTION_STR set
* GEMINI_API_KEY set

```
GEMINI_API_KEY=your-key
POSTGRES_CONNECTION_STR="postgres://root:admin@localhost/mydb"
MODEL_EMBED_NAME=qwen3-embedding:0.6b
```

***
* build

```
cargo build
```

***
data file path: ./data

***
* table: table.sql

```
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE documents (
  id SERIAL PRIMARY KEY,
  content TEXT NOT NULL,
  embedding vector(1024)
);

```
***
* vector add
```
target\debug\mcp_19.exe create
```

* RAG search
```
target\debug\mcp_19.exe search
```

***
### blog

https://zenn.dev/knaka0209/scraps/8f794f1292b323

***


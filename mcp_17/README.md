# mcp_17

 Version: 0.9.1

 date    : 2025/11/20

 update :

***

Rust Ollama , RAG Search

* model-embed: embeddinggemma Ollama
* model: gemini-2.0-flash
* Postgres sqlx use

***
### setup

* build

```
cargo build
cargo run
```

***
* .env
* POSTGRES_CONNECTION_STR set
* GEMINI_API_KEY set

```
POSTGRES_CONNECTION_STR="postgres://postgres:admin@localhost/postgres"
GEMINI_API_KEY=your-key
MODEL_EMBED_NAME=embeddinggemma

```

***


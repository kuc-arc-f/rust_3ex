# mcp_13

 Version: 0.9.1

 date    : 2025/11/14

 update :

***

Rust , RAG Search

* model-embed: gemini-embedding-001
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
* src/main.rs
* POSTGRES_CONNECTION_STR set
* GEMINI_API_KEY set

```
static POSTGRES_CONNECTION_STR: &str = "postgres://postgres:admin@localhost/postgres";
static GEMINI_API_KEY: &str = "your-key"
```

***


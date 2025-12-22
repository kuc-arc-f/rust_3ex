# mcp_23

 Version: 0.9.1

 date    : 2025/12/20

 update : 2025/12/22

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
* vector data add
```
cargo build
target\debug\mcp_23.exe create
```
***
* RAG search
```
target\debug\mcp_23.exe search
```
***
* data path: ./data

***
* table: table.sql

***




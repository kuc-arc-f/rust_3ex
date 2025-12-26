# mcp_27

 Version: 0.9.1

 date    : 2025/12/26

 update :

***

Rust RAG Search , Qdrant

* embedding: gemini-embedding-001
* gemma3-27b

***
* .env
```
GEMINI_API_KEY=your-key
```
***
* build

```
cargo build
```
***
* init, collection add
```
target\debug\mcp_27.exe init
```
* vector data add
```
target\debug\mcp_27.exe create
```
* RAG search
```
target\debug\mcp_27.exe search
```


***
* data path: ./data

***





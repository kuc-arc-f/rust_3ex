# mcp_9

 Version: 0.9.1

 date    : 2025/10/03 

 update :

***

node.js + Rust , MCP server

* Turso libsql use , JSON-RPC 2.0

***
### setup

* build
```
cargo build --release
```

***
* src/main.rs
* TURSO_DATABASE_URL, TURSO_AUTH_TOKEN set

```
static TURSO_DATABASE_URL: &str = "";
static TURSO_AUTH_TOKEN: &str = "";
```

***
* table: scheme.sql

***
### Test

* test_create.js

***

# mcp_7

 Version: 0.9.1

 date    : 2025/10/28 

 update :

***

Rust MCP server

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

```
CREATE TABLE IF NOT EXISTS item_price (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  data TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

***
### Test

* test-code: test_list_xls.js

***


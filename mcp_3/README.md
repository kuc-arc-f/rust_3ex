# mcp_3

 Version: 0.9.1

 date    : 2025/10/03 

 update :

***

Rust + Turso libsql , MCP server

***
### setup

* build
```
cargo build --release
```
***
### Prompt
```
コーヒー , 110　円を購入。品名、価格 の値をAPIに送信して欲しい。
```

***
* settings.json , GEMINI-CLI
* TURSO_DATABASE_URL, TURSO_AUTH_TOKEN set

```
    "rust_mcp_server_3": {
      "command": "/path/mcp_3/target/release/rust_mcp_server_3.exe",
      "env": {
        "TURSO_DATABASE_URL": "",
        "TURSO_AUTH_TOKEN": ""
      }
    }

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


# mcp_2

 Version: 0.9.1

 date    : 2025/10/03 

 update :

***

Rust turso, MCP example

***
### setup

* build
```
cargo build --release
```
***
### Prompt
```
コーヒー , 170　円を購入。品名、価格 の値をAPIに送信して欲しい。
```

***
* settings.json , GEMINI-CLI

```
"mcpServers": {
    "rust_mcp_server_2": {
      "command": "/path/mcp_2/target/release/rust_mcp_server_2.exe",
      "env": {
        "HOGE": ""
      }
    }    
}

```

***


# mcp_5

 Version: 0.9.1

 date    : 2025/10/03 

 update :

***

Rust , MCP Server

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
* test-code: javascript

```js
import { spawn } from "child_process";

class RpcClient {
  constructor(command) {
    this.proc = spawn(command);
    this.idCounter = 1;
    this.pending = new Map();

    this.proc.stdout.setEncoding("utf8");
    this.proc.stdout.on("data", (data) => this._handleData(data));
    this.proc.stderr.on("data", (err) => console.error("Rust stderr:", err.toString()));
    this.proc.on("exit", (code) => console.log(`Rust server exited (${code})`));
  }

  _handleData(data) {
    // 複数行対応
    data.split("\n").forEach((line) => {
      if (!line.trim()) return;
      try {
        //console.log(line);
        const msg = JSON.parse(line);
        if (msg.id && this.pending.has(msg.id)) {
          const { resolve } = this.pending.get(msg.id);
          this.pending.delete(msg.id);
          resolve(msg.result);
        }
      } catch (e) {
        //console.error("JSON parse error:", e, line);
      }
    });
  }

  call(method, params = {}) {
    const id = this.idCounter++;
    const payload = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };

    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.proc.stdin.write(JSON.stringify(payload) + "\n");
    });
  }

  close() {
    this.proc.kill();
  }
}

// -----------------------------
// 実行例
// -----------------------------
async function main() {
  const client = new RpcClient("/path/target/release/rust_mcp_server_5.exe");

  const result1 = await client.call(
    "tools/call", 
    { 
      name: "purchase_list", 
      arguments: null, 
    },
  );
  console.log("add結果:", result1);

  client.close();
  try{
    if(result1.content[0]){
      console.log(result1.content[0].text);

    }
  }catch(e){ console.error(e);}
}

main().catch(console.error);


```
***


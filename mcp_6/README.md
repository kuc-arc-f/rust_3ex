# mcp_6

 Version: 0.9.1

 date    : 2025/10/26 

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
* test-code

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
      console.log("line=", line);
      try {
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
  const client = new RpcClient("/work/rust/extra/mcp_6/target/release/rust_mcp_server_6.exe");

  const resp = await client.call(
    "tools/call", 
    { 
      name: "purchase_list_excel", 
      arguments: {
        template_purchase: "/work/node/extra/mcp/mcp_server/mcp_cli_6/public/input.xlsx",
        xls_out_dir:"/work/node/extra/mcp/mcp_server/mcp_cli_6/public/data",
      }, 
    },

  );
  console.log("add結果:", resp);

  client.close();
  try{
    if(resp.content[0]){
      console.log(resp.content[0].text)
    }else{
      console.log("NG")
    }

  }catch(e){console.log(e)}
}

main().catch(console.error);

```
***


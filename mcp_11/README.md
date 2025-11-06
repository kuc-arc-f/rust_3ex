# mcp_11

 Version: 0.9.1

 date    : 2025/11/05

 update : 2025/11/06

***

node.js + Rust , MCP server

* Postgres sqlx use

***
### setup

* build
```
cargo build --release
```

***
* src/main.rs
* POSTGRES_CONNECTION_STR set

```
static POSTGRES_CONNECTION_STR: &str = "postgres://postgres:admin@localhost/postgres";
```

***
* table: scheme.sql

***
### Test
* test: test_create.js

```
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
  const client = new RpcClient("/home/naka/work/rust/extra/mcp_11/target/release/rust_mcp_server_11");
  
  const result1 = await client.call(
    "tools/call", 
    { 
      name: "test_create", 
      arguments:{
        title: "t1-1106",
        content: "c1",
      }      
    },
  );
  console.log("add結果:", result1);

  client.close();
}

main().catch(console.error);

```

***
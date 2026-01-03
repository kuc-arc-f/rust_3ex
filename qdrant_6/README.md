# qdrant_6

 Version: 0.9.1

 date    : 2026/01/03

 update :

***

Rust RAG Search , MCP Server Qdrant

* Agent Skills use
* embedding: gemini-embedding-001

***

* vecctor data add

https://github.com/kuc-arc-f/rust_3ex/tree/main/mcp_27

***

* src/mod_config.rs
* API_KEY: gemini API KEY

```
pub static API_KEY: &str = "your-key";
```

***
* build

```
cargo build
```
***
* ts-code: test3.js
* JSON-RPC 2.0

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
      //console.log("line=", line)
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
  const client = new RpcClient("/work/rust/extra/qdrant_6/target/debug/rust_qdrant_6.exe");
  
  const result1 = await client.call(
    "tools/call", 
    { 
      name: "rag_search", 
      arguments:{
        query: "二十四節季",
      }      
    },
  );
  client.close();
  console.log("resp=", result1);
  if(result1.content[0].text){
    console.log("text:"+ result1.content[0].text);
  }
}

main().catch(console.error);

```

***
### blog

https://zenn.dev/knaka0209/scraps/02fda731283c1a






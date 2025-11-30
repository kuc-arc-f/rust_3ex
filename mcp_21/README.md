# mcp_21

 Version: 0.9.1

 date    : 2025/11/30

 update :

***

Rust MCP Server , RAG Search , Ollama 

* model-embed: qwen3-embedding:0.6b
* Postgres pgvector use
* GEMINI-CLI use

***
* vector data add

https://github.com/kuc-arc-f/rust_3ex/tree/main/mcp_19

***
* table: table.sql

***
* build

```
cargo build --release
```

***
* test-code: test_list.js

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
      //console.log("line=", line);
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
  const client = new RpcClient("/work/rust/extra/mcp_21/target/release/rust_mcp_server_21.exe");
  const resp = await client.call(
    "tools/call", 
    { 
      name: "rag_search", 
      arguments:{
        input_text: "二十四節季",
        pg_connet_str: "postgres://root:admin@localhost/mydb",
      }       
    },

  );
  client.close();
  
  try{
    if(resp.content[0].text){
      console.log(resp.content[0].text)
    }

  }catch(e){console.log(e)}
}

main().catch(console.error);

```

***


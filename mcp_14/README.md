# mcp_14

 Version: 0.9.1

 date    : 2025/11/14

 update :

***

Rust , RAG Search MCP Server

* model-embed: gemini-embedding-001
* model: gemini-2.0-flash
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
* GEMINI_API_KEY set

```
static POSTGRES_CONNECTION_STR: &str = "postgres://postgres:admin@localhost/postgres";
static GEMINI_API_KEY: &str = "your-key"
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
      console.log("line=", line);
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
  const client = new RpcClient("/home/naka/work/rust/extra/mcp_14/target/release/rust_mcp_server_14");
  const resp = await client.call(
    "tools/call", 
    { 
      name: "rag_search", 
      arguments:{
        input_text: "二十四節気",
      }       
    },

  );
  //console.log("add結果:", resp);

  client.close();
  try{
    if(resp.content[0].text){
      const json = JSON.parse(resp.content[0].text)
      if(json.candidates[0] && json.candidates[0].content.parts[0]){
        console.log("text=", json.candidates[0].content.parts[0].text)
      }
    }

  }catch(e){console.log(e)}
}

main().catch(console.error);

```

***


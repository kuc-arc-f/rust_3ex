use libsql::Database;
use libsql::Builder;
use libsql::Connection;
use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
use dotenvy::dotenv;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct AddTenParams {
    value: i32,
}

#[derive(Debug, Deserialize,Serialize)]
struct PurchaseParams {
    name: String,
    price: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    id: i64,
    //content: String,
    data: String,
    created_at: String,
    updated_at: String,
}


fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}

static TURSO_DATABASE_URL: &str = "";
static TURSO_AUTH_TOKEN: &str = "";


/**
*
* @param
*
* @return
*/
async fn purchase_handler(params: Value, request_id: Option<Value>) -> JsonRpcResponse {
    let url = TURSO_DATABASE_URL.to_string();
    let token = TURSO_AUTH_TOKEN.to_string();
    println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();  
    
    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "purchase_list" {
            let order_sql = "ORDER BY created_at DESC LIMIT 5;";
            let sql = format!("SELECT id, data ,created_at, updated_at 
            FROM item_price
            {}
            "
            , order_sql
            );
            println!("sql={}", sql);
            let mut rows = conn.query(&sql,
                (),  // 引数なし
            ).await.unwrap();
            let mut todos: Vec<Item> = Vec::new();
            while let Some(row) = rows.next().await.unwrap() {
                let id: i64 = row.get(0).unwrap();
                let data: String = row.get(1).unwrap();
                todos.push(Item {
                    id: id,
                    data: data,
                    created_at: row.get(2).unwrap(),
                    updated_at: row.get(3).unwrap(),        
                });        
            }
            let json_string_variable = serde_json::to_string(&todos).expect("JSON convert error");
            println!("変換されたJSON文字列: {}", json_string_variable);            

            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": json_string_variable.to_string()
                        }
                    ]
                })),
                error: None,
            };    
        }
    }
    return JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "Tool not found".to_string(),
        }),
    };    
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "rust-purchase-list-server",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "purchase_list",
//                        "description": "品名と価格を受け取り、値をAPIに送信します。",
                        "description": "購入品リストを、表示します。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                            },
                            "required": []
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            if let Some(params) = request.params {
                purchase_handler(params, request.id).await
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Tool not found".to_string(),
                    }),
                }
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
            }),
        },
    }
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    eprintln!("MCP Server started. Waiting for requests...");

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                if input.trim().is_empty() {
                    continue;
                }

                match serde_json::from_str::<JsonRpcRequest>(&input) {
                    Ok(request) => {
                        eprintln!("Received request: {:?}", request);
                        let response = handle_request(request).await;
                        let response_json = serde_json::to_string(&response).unwrap();
                        eprintln!("Sending response: {}", response_json);
                        writeln!(stdout, "{}", response_json).unwrap();
                        stdout.flush().unwrap();
                    }
                    Err(e) => {
                        eprintln!("Failed to parse request: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32700,
                                message: format!("Parse error: {}", e),
                            }),
                        };
                        let response_json = serde_json::to_string(&error_response).unwrap();
                        writeln!(stdout, "{}", response_json).unwrap();
                        stdout.flush().unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

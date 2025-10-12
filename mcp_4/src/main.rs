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


fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}

static TURSO_DATABASE_URL: &str = "";
static TURSO_AUTH_TOKEN: &str = "";

async fn purchase_handler(params: Value, request_id: Option<Value>) -> JsonRpcResponse {
    let url = TURSO_DATABASE_URL.to_string();
    let token = TURSO_AUTH_TOKEN.to_string();
    //println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();    

    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "purchase" {
            if let Some(arguments) = params.get("arguments") {
                match serde_json::from_value::<PurchaseParams>(arguments.clone()) {
                    Ok(purchase_params) => {
                        let post_data = PurchaseParams {
                            name: purchase_params.name.clone(),
                            price: purchase_params.price
                        };    
                        let json_string_variable = serde_json::to_string(&post_data).expect("JSON convert error");
                        println!("変換されたJSON文字列: {}", json_string_variable); 
                        let sql = format!("INSERT INTO item_price (data) VALUES ('{}')", &json_string_variable);
                        let mut result = conn
                            .execute(&sql, ())
                            .await
                            .unwrap();

                        let result = purchase(purchase_params.name, purchase_params.price);
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request_id,
                            result: Some(json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": result
                                    }
                                ]
                            })),
                            error: None,
                        };                       

                    }
                    Err(e) => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request_id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: format!("Invalid parameters: {}", e),
                            }),
                        };
                    }
                }
            }
        }
    }
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "Tool not found".to_string(),
        }),
    }
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "rust-purchase-server",
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
                        "name": "purchase",
                        "description": "品名と価格を受け取り、値をAPIに送信します。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": {
                                    "type": "string",
                                    "description": "購入する品名"
                                },
                                "price": {
                                    "type": "number",
                                    "description": "価格"
                                }
                            },
                            "required": ["name", "price"]
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

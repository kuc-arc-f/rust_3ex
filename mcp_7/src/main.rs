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
pub struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct AddTenParams {
    value: i32,
}

#[derive(Debug, Deserialize,Serialize)]
pub struct PurchaseParams {
    name: String,
    price: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    id: i64,
    data: String,
    created_at: String,
    updated_at: String,
}


fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}

static TURSO_DATABASE_URL: &str = "";
static TURSO_AUTH_TOKEN: &str = "";

mod mod_purchase;

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
                    },
                    {
                        "name": "purchase_list",
                        "description": "購入品リストを、表示します。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                            },
                            "required": []
                        }
                    },
                    {
                        "name": "purchase_list_excel",
                        "description": "購入品リスト、ダウンロード リンク、表示します。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                            },
                            "required": []
                        }
                    },                                        
                    {
                        "name": "purchase_delete",
                        "description": "指定ID 値を受け取り、購入データを削除します。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "number",
                                    "description": "ID"
                                }
                            },
                            "required": ["id"]
                        }
                    },

                ]
            })),
            error: None,
        },
        "tools/call" => {
            if let Some(params) = request.params {
                if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                    if tool_name == "purchase" {
                        mod_purchase::purchase_handler(params, request.id).await
                    } else if tool_name == "purchase_list"{
                        mod_purchase::purchase_list_handler(params, request.id).await
                    } else if tool_name == "purchase_delete"{
                        mod_purchase::purchase_delete_handler(params, request.id).await
                    } else if tool_name == "purchase_list_excel"{
                        mod_purchase::purchase_list_excel_handler(params, request.id).await
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
                } else{
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
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "arguments.name not found".to_string(),
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

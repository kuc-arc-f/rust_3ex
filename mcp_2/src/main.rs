use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

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
const API_ENDPOINT: &'static str = "http://localhost:8787/api/data/create";

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
                        //"description": "品名と価格を受け取り、購入処理を模倣します",
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
                if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                    if tool_name == "purchase" {
                        if let Some(arguments) = params.get("arguments") {
                            match serde_json::from_value::<PurchaseParams>(arguments.clone()) {
                                Ok(purchase_params) => {
                                    let client = reqwest::Client::new();
                                    let post_data = PurchaseParams {
                                        name: purchase_params.name.clone(),
                                        price: purchase_params.price
                                    };    
                                    let json_string_variable = serde_json::to_string(&post_data).expect("JSON convert error");
                                    println!("変換されたJSON文字列: {}", json_string_variable);                         
                                    let send_data = json!({
                                        "content": "item_price",
                                        "data": &json_string_variable
                                    });
                                    match client.post(API_ENDPOINT).json(&send_data).send().await {
                                        Ok(api_res) => {
                                            if api_res.status().is_success() {
                                                let result = purchase(purchase_params.name, purchase_params.price);
                                                return JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    id: request.id,
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
                                            } else {
                                                return JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    id: request.id,
                                                    result: None,
                                                    error: Some(JsonRpcError {
                                                        code: -32000,
                                                        message: format!("API request failed with status: {}", api_res.status()),
                                                    }),
                                                };
                                            }
                                        }
                                        Err(e) => {
                                            return JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                result: None,
                                                error: Some(JsonRpcError {
                                                    code: -32000,
                                                    message: format!("API request failed: {}", e),
                                                }),
                                            };
                                        }
                                    }
                                }
                                Err(e) => {
                                    return JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
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
            }
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
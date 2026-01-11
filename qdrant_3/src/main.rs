use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

mod mod_search;
mod mod_config;
static MODEL_NAME: &str = "models/gemini-embedding-001";
static COLLECT_NAME: &str = "document-2";
static EMBED_SIZE: u64 =3072;
static API_KEY: &str = mod_config::API_KEY;

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
#[derive(Debug, Deserialize)]
struct RagSearchParams {
    query: String,
}

fn add_ten(value: i32) -> i32 {
    value + 10
}

async fn rag_search(query: String) -> String {
    let resp = mod_search::CheckSimalirity(query).await;
    let send_text = format!("日本語で、回答して欲しい。\n{}", resp);
    println!("send_text={}\n", send_text);

    return send_text;
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "rust-add-ten-server",
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
                        "name": "add_ten",
                        "description": "入力値に10を加算して返却します",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "value": {
                                    "type": "number",
                                    "description": "加算する元の数値"
                                }
                            },
                            "required": ["value"]
                        }
                    },
                    {
                        "name": "rag_search",
                        "description": "検索文字から、RAG検索 結果を返す。",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "検索文字"
                                }
                            },
                            "required": ["query"]
                        }
                    }

                ]
            })),
            error: None,
        },
        "tools/call" => {
            if let Some(params) = request.params {
                if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                    if tool_name == "add_ten" {
                        if let Some(arguments) = params.get("arguments") {
                            match serde_json::from_value::<AddTenParams>(arguments.clone()) {
                                Ok(add_params) => {
                                    let result = add_ten(add_params.value);
                                    return JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
                                        result: Some(json!({
                                            "content": [
                                                {
                                                    "type": "text",
                                                    "text": format!("入力値: {}, 結果: {}", add_params.value, result)
                                                }
                                            ]
                                        })),
                                        error: None,
                                    };
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
                    if tool_name == "rag_search" {
                        if let Some(arguments) = params.get("arguments") {
                            match serde_json::from_value::<RagSearchParams>(arguments.clone()) {
                                Ok(add_params) => {
                                    let result = rag_search(add_params.query).await;
                                    return JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
                                        result: Some(json!({
                                            "content": [
                                                {
                                                    "type": "text",
                                                    "text": format!("結果: {}",  result)
                                                }
                                            ]
                                        })),
                                        error: None,
                                    };
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
/**
*
* @param
*
* @return
*/
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

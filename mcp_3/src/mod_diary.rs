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
struct AddTenParams {
    value: i32,
}

#[derive(Debug, Deserialize,Serialize)]
struct DiaryParams {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    id: i64,
    data: String,
    created_at: String,
    updated_at: String,
}
#[derive(Debug, Deserialize,Serialize)]
struct PurchaseDeleteParams {
    id: i32,
}


pub fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}


pub async fn diary_add_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{

    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "diary_add" {
            dotenv().ok();
            let url = env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
            let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");
            //let url = TURSO_DATABASE_URL.to_string();
            //let token = TURSO_AUTH_TOKEN.to_string();
            println!("TURSO_DATABASE_URL={}", url);
            let db = Builder::new_remote(url, token).build().await.unwrap();
            let conn = db.connect().unwrap();    

            if let Some(arguments) = params.get("arguments") {
                match serde_json::from_value::<DiaryParams>(arguments.clone()) {
                    Ok(purchase_params) => {
                        let post_data = DiaryParams {
                            text: purchase_params.text.clone(),
                        };    
                        let json_string_variable = serde_json::to_string(&post_data).expect("JSON convert error");
                        println!("変換されたJSON文字列: {}", json_string_variable); 
                        let sql = format!("INSERT INTO mcp_diary (data) VALUES ('{}')", &json_string_variable);
                        let mut result = conn
                            .execute(&sql, ())
                            .await
                            .unwrap();

                        return super::JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request_id,
                            result: Some(json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": "OK".to_string()
                                    }
                                ]
                            })),
                            error: None,
                        };                       

                    }
                    Err(e) => {
                        return super::JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request_id,
                            result: None,
                            error: Some(super::JsonRpcError {
                                code: -32602,
                                message: format!("Invalid parameters: {}", e),
                            }),
                        };
                    }
                }
            }
        }
    }
    super::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: None,
        error: Some(super::JsonRpcError {
            code: -32601,
            message: "Tool not found".to_string(),
        }),
    }
}


/**
*
* @param
*
* @return
*/
pub async fn diary_list_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Deserialize)]
    struct ItemData {
        text: String,
    }

    let url = env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
    let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");
    //let url = TURSO_DATABASE_URL.to_string();
    //let token = TURSO_AUTH_TOKEN.to_string();
    println!("TURSO_DATABASE_URL={}", url);            
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();  
    
    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "diary_list" {
            let order_sql = "ORDER BY created_at DESC LIMIT 5;";
            let sql = format!("SELECT id, data ,created_at, updated_at 
            FROM mcp_diary
            {}
            "
            , order_sql
            );
            println!("sql={}", sql);
            let mut rows = conn.query(&sql,
                (),  // 引数なし
            ).await.unwrap();
            let mut todos: Vec<Item> = Vec::new();
            let mut out_str: String = "".to_string();
            while let Some(row) = rows.next().await.unwrap() {
                let id: i64 = row.get(0).unwrap();
                let data: String = row.get(1).unwrap();
                todos.push(Item {
                    id: id,
                    data: data.clone(),
                    created_at: row.get(2).unwrap(),
                    updated_at: row.get(3).unwrap(),        
                });  
                let row_item: ItemData = serde_json::from_str(&data).expect("data JSON decord error");
                println!("デコードされた構造体: {:?}", row_item);  
                let row_str: String = format!("id: {}\n{}\n", id, row_item.text);    
                println!("row_str: {:?}", row_str); 
                out_str = format!("{}{}", &out_str, &row_str); 
            }

            return super::JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": out_str.to_string()
                        }
                    ]
                })),
                error: None,
            };    
        }
    }
    return super::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: None,
        error: Some(super::JsonRpcError {
            code: -32601,
            message: "Tool not found".to_string(),
        }),
    };    
}

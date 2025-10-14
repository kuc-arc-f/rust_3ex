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
struct PurchaseParams {
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


pub fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}

pub async fn purchase_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{

    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "purchase" {
            let url = super::TURSO_DATABASE_URL.to_string();
            let token = super::TURSO_AUTH_TOKEN.to_string();
            //println!("TURSO_DATABASE_URL={}", url);
            let db = Builder::new_remote(url, token).build().await.unwrap();
            let conn = db.connect().unwrap();    

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
                        return super::JsonRpcResponse {
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
pub async fn purchase_list_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
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

            return super::JsonRpcResponse {
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
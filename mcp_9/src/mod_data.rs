use libsql::Database;
use libsql::Builder;
use libsql::Connection;
use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
use dotenvy::dotenv;

#[derive(Debug, Deserialize,Serialize)]
struct ItemParams {
    content: String,
    data: String,
}
#[derive(Debug, Deserialize,Serialize)]
struct ItemListParams {
    content: String,
}
#[derive(Debug, Deserialize,Serialize)]
struct ItemGetParams {
    content: String,
    id: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    id: i64,
    data: String,
    created_at: String,
    updated_at: String,
}
#[derive(Debug, Deserialize,Serialize)]
struct ItemDeleteParams {
    content: String,
    id: i32,
}

pub fn purchase(product_name: String, price: i32) -> String {
    format!("「{}」を{}円で購入しました。", product_name, price)
}

pub async fn data_create_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    //println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();    

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemParams>(arguments.clone()) {
            Ok(item_params) => {
                let post_data = ItemParams {
                    content: item_params.content.clone(),
                    data: item_params.data.clone()
                };    
                let sql = format!("INSERT INTO {} (data) VALUES ('{}')", &post_data.content, &post_data.data);
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
pub async fn data_getone_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{

    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();  
    
    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemGetParams>(arguments.clone()) {
          Ok(item_get_params) => {
            let content = item_get_params.content.clone();
            let id_value = item_get_params.id;
            let order_sql = "ORDER BY created_at DESC LIMIT 5;";
            let sql = format!("SELECT id, data ,created_at, updated_at 
            FROM {} WHERE ID = {}
            "
            , content, id_value
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

/**
*
* @param
*
* @return
*/
pub async fn data_list_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();  
    
    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemListParams>(arguments.clone()) {
          Ok(item_list_params) => {
            let content = item_list_params.content.clone();
            let order_sql = "ORDER BY created_at DESC LIMIT 10;";
            let sql = format!("SELECT id, data ,created_at, updated_at 
            FROM {} {}
            "
            , content, order_sql
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

/**
*
* @param
*
* @return
*/
pub async fn data_delete_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{

    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    //println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();    

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemDeleteParams>(arguments.clone()) {
            Ok(item_delete_params) => {
                let id_value = item_delete_params.id;
                let content_value = item_delete_params.content.clone();
                //select-id
                let select_sql = format!("SELECT id, data ,created_at, updated_at 
                FROM {}
                WHERE id= {} ;
                "
                ,content_value , id_value
                );
                let mut select_rows = conn.query(&select_sql,
                    (),  // 引数なし
                ).await.unwrap(); 
                let mut count = 0;
                while let Some(row) = select_rows.next().await.unwrap() {
                    count += 1;
                }
                if count == 0 {
                    return super::JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request_id,
                        result: None,
                        error: Some(super::JsonRpcError {
                            code: -32602,
                            message: format!("Invalid parameters, id={}", id_value),
                        }),
                    };                            
                }

                let sql = format!("DELETE FROM {} WHERE id = {}"
                ,content_value , id_value);
                let mut result = conn
                    .execute(&sql, ())
                    .await
                    .unwrap();

                let resp = format!("Complete delete, id={}", id_value);
                return super::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": resp
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
pub async fn data_update_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Deserialize,Serialize)]
    struct UpdateParams {
        content: String,
        data: String,
        id: i32,
    }    
    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    //println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();    

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<UpdateParams>(arguments.clone()) {
            Ok(item_params) => {
                /*
                let post_data = UpdateParams {
                    content: item_params.content.clone(),
                    data: item_params.data.clone()
                    id: item_params.id,
                };    
                */
                let sql = format!("UPDATE {} SET data = '{}' WHERE id = {}"
                , &item_params.content, &item_params.data, &item_params.id
              );
//                let sql = format!("INSERT INTO {} (data) VALUES ('{}')", &post_data.content, &post_data.data);
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
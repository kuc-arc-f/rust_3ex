use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
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
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}
#[derive(Debug, Deserialize,Serialize)]
struct ItemDeleteParams {
    content: String,
    id: i32,
}

/**
*
* @param
*
* @return
*/
pub async fn test_create_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Deserialize,Serialize)]
    struct TestParams {
        title: String,
        content: String,
    }    
    let con_str = super::POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");    

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<TestParams>(arguments.clone()) {
            Ok(item_params) => {
                let title_str = item_params.title.clone();
                let content_str = item_params.content.clone();
                let result = sqlx::query(
                    "INSERT INTO test (title, content) VALUES ($1, $2) RETURNING id",
                )
                .bind(&title_str)
                .bind(&content_str)
                .execute(&pool)
                .await.unwrap();

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
pub async fn test_list_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Serialize, Deserialize)]
    pub struct OutListItem {
        id: i64,
        title: String,
        content: String,
    }    
    let con_str = super::POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");       
    
    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemListParams>(arguments.clone()) {
          Ok(item_list_params) => {
            let content = item_list_params.content.clone();
            let order_sql = "ORDER BY created_at DESC LIMIT 10;";
            //let sql = format!("SELECT id, title, content ,created_at, updated_at 
            let sql = format!("SELECT id, title, content  
            FROM {} {}
            "
            , content, order_sql
            );
            println!("sql={}", sql);
            let rows = sqlx::query(&sql)
                .fetch_all(&pool)
                .await.unwrap();
            let todoItems: Vec<OutListItem> = rows
                .into_iter()
                .map(|row| OutListItem {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                })
                .collect();           
            let out = serde_json::to_string(&todoItems).unwrap();
            return super::JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": out.to_string()
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
pub async fn test_delete_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DeleteListItem {
        id: i64,
        title: String,
        content: String,
    }
    let con_str = super::POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");  

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ItemDeleteParams>(arguments.clone()) {
            Ok(item_delete_params) => {
                let id_value = item_delete_params.id;
                let content_value = item_delete_params.content.clone();

                let sql = format!("SELECT id, title, content  
                FROM {}
                WHERE id = {}
                "
                , content_value , id_value
                );
                println!("sql={}", sql);
                let rows = sqlx::query(&sql)
                    .fetch_all(&pool)
                    .await.unwrap();
                let mut count = 0;
                let todos: Vec<DeleteListItem> = rows
                .into_iter()
                .map(|row| DeleteListItem {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                })
                .collect();
                count = todos.len();
                println!("Vecに含まれるデータの件数: {}", count);                 

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
                println!("sql={}", &sql);

                let result = sqlx::query(&sql)
                    .execute(&pool)
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
pub async fn test_update_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Deserialize,Serialize)]
    struct UpdateParams {
        content: String,
        title: String,
        id: i32,
    }    
    let con_str = super::POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");

    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<UpdateParams>(arguments.clone()) {
            Ok(item_params) => {
                let sql = format!("UPDATE test SET title = '{}' , content='{}' WHERE id = {}"
                , &item_params.title, &item_params.content , &item_params.id
               );
                let result = sqlx::query(&sql)
                .execute(&pool)
                .await.unwrap();
                println!("# /api/update END");

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


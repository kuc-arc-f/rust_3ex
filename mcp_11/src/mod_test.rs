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

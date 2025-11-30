use pgvector::Vector;
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use std::fmt;
use std::env;
use std::path::Path;
use std::io::{self, BufRead, Read, Write};
use dotenvy::dotenv;

#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}
#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize,Serialize)]
struct RagSearchParams {
    input_text: String,
    pg_connet_str: String,
}

/**
*
* @param
*
* @return
*/
pub async fn EmbedUserQuery(query :String) -> Vec<f32> {
    let items : Vec<f32> = Vec::new();
    let client = reqwest::Client::new();

    let request = EmbeddingRequest {
        model: "qwen3-embedding:0.6b".to_string(),
        prompt: query.to_string(),
    };

    let res = client
        .post("http://localhost:11434/api/embeddings")
        .json(&request)
        .send()
        .await.unwrap();
    println!("Status: {:?}", res.status());

    if res.status().is_success() {
        let response_body: EmbeddingResponse = res.json().await.unwrap();
        println!("Embedding length: {}", response_body.embedding.len());
        // Print first few elements to verify without flooding console
        if response_body.embedding.len() > 0 {
             println!("dimensions.len: {}", response_body.embedding.len());
             return response_body.embedding;
         } else {
             println!("Embedding: {:?}", response_body.embedding);
        }
       
    } else {
        println!("Request failed: {:?}", res.status());
        let text = res.text().await.unwrap();
        println!("Response text: {}", text);
    }

   return items;
}


/**
*
* @param
*
* @return
*/
async fn CheckSimalirity(query: String, pg_connet_str: String) -> String {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EmbedItem {
        name: String,
        content: String,
        embeddings: Vec<u8>
    }
    let input_f32 = EmbedUserQuery(query.clone()).await;
    //println!("input_f32.len={}", input_f32.len());
    let query_vec = Vector::from(input_f32);

    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&pg_connet_str).await.expect("Failed to create pool");      
    let rows = sqlx::query(
        "SELECT id, content , embedding
         FROM documents
         ORDER BY embedding <=> $1
         LIMIT 3"
    )
    .bind(&query_vec)
    .fetch_all(&pool)
    .await.unwrap();
    let mut matches : String = "".to_string();
    for row in rows {
        let id: i32 = row.get("id");
        let content: String = row.get("content");
        matches.push_str(&content.clone());
        println!("ID: {}, cont.len={}", id, content.len() );
    }

    let mut out_str : String = "".to_string();
    if matches.len() > 0 {
        out_str = format!("context: {}\n\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    }
    return out_str.to_string();
}


/**
*
* @param
*
* @return
*/
pub async fn rag_search_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    #[derive(Debug, Serialize, Deserialize)]
    pub struct OutListItem {
        id: i64,
        title: String,
        content: String,
    }    
    
    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<RagSearchParams>(arguments.clone()) {
          Ok(item_list_params) => {
            let input_text = item_list_params.input_text.clone();
            let pg_connet_str = item_list_params.pg_connet_str.clone();
            println!("pg_connet_str={}\n", pg_connet_str);
            let pool = PgPoolOptions::new().max_connections(5)
            .connect(&pg_connet_str).await.expect("Failed to create pool");       

            let input = CheckSimalirity(input_text, pg_connet_str).await;
            let send_text = format!("日本語で、回答して欲しい。\n{}", input);
            println!("send_text={}\n", send_text);
            let mut out_text: String = input.clone();

            return super::JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": out_text.to_string()
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
fn print_type_of<T>(_: &T) {
    println!("Type: {}", std::any::type_name::<T>());
}

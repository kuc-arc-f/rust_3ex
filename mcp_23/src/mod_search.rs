use dotenvy::dotenv;
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
use std::fs;
use std::path::Path;
use std::io::{self, Read};


#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}


// エンベディング結果を格納する構造体
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
}


/**
*
* @param
*
* @return
*/
pub async fn EmbedUserQuery(query :String) -> Vec<f32> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    println!("api_key={}", api_key);

    let items : Vec<f32> = Vec::new();
    let body = json!({
      "model": "models/gemini-embedding-001".to_string(),
      "content": {"parts":[{"text": query.to_string()}]},
      //"output_dimensionality": 1024
    });

    let items : Vec<f32> = Vec::new();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&api_key).unwrap());
    let send_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent".to_string();
    
    // --- POST 送信 ---
    let res = client
        .post(&send_url)
        .headers(headers)
        .json(&body)
        .send()
        .await.unwrap();

    println!("Status: {:?}", res.status());
    if res.status().is_success() {
        let response_body: Value = res.json().await.unwrap();
        
        // エンベディングデータを取得
        let embed_values = response_body["embedding"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();
        
        print!("[0]=");
        //print_type_of(&embed_values[0]);
        println!("    エンベディング次元数: {}", embed_values.len());
        println!("    最初の5要素: {:?}", &embed_values[..embed_values.len().min(5)]);
        return embed_values;
    } else {
        println!("Error: {:?}", res.text().await.unwrap());
    }

    return items;
}

/**
*
* @param
*
* @return
*/
pub async fn CheckSimalirity(query: String) -> String {
    dotenv().ok();
    let con_str = env::var("POSTGRES_CONNECTION_STR").expect("POSTGRES_CONNECTION_STR must be set");

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EmbedItem {
        name: String,
        content: String,
        embeddings: Vec<u8>
    }
    let input_f32 = EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());
    let query_vec = Vector::from(input_f32);

    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");      
    
    // コサイン距離による検索
    let rows = sqlx::query(
        "SELECT id, content , embedding
         FROM documents
         ORDER BY embedding <=> $1
         LIMIT 2"
    )
    .bind(&query_vec)
    .fetch_all(&pool)
    .await.unwrap();

    println!("\nコサイン距離による類似検索結果:");
    let mut matches : String = "".to_string();
    for row in rows {
        let id: i32 = row.get("id");
        let content: String = row.get("content");
        matches.push_str(&content.clone());
        //let cosine_distance: f32 = row.get("cosine_distance");
        println!("ID: {}, cont.len={}", id, content.len() );
    }
    
    let mut out_str : String = "".to_string();
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    }
    return out_str.to_string();
}
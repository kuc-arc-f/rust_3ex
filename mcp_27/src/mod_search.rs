use dotenvy::dotenv;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, ScalarQuantizationBuilder,
    SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant, QdrantError};
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    let clientQdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EmbedItem {
        name: String,
        content: String,
        embeddings: Vec<u8>
    }
    let input_f32 = EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());
    let search_result = clientQdrant
        .search_points(
            SearchPointsBuilder::new(super::COLLECT_NAME, input_f32, 2)
                .with_payload(true)
                .params(SearchParamsBuilder::default().exact(true)),
        )
        .await.unwrap();
    //dbg!(&search_result);

    let resplen = search_result.result.len();
    println!("#list-start={}", resplen);
    println!("\nコサイン距離による類似検索結果:");
    let mut matches : String = "".to_string();
    let mut out_str : String = "".to_string();
    for row_resp in &search_result.result {
        let content = &row_resp.payload["content"];
        let content_str = format!("{}\n\n", content);
        matches.push_str(&content_str.clone().to_string());
    }
    //println!("matches={}\n", &matches);
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    }

    return out_str.to_string();
}
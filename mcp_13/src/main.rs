use bytemuck::{cast_slice, Pod, Zeroable};
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;

//use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::io::{self, Read};

use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

static POSTGRES_CONNECTION_STR: &str = "";
static GEMINI_API_KEY: &str = "";
static MODEL_NAME: &str = "models/gemini-embedding-001";

#[derive(Debug)]
struct VectorLengthError;

impl fmt::Display for VectorLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vectors must have the same length")
    }
}
impl std::error::Error for VectorLengthError {}

/// テキストをチャンクに分割する構造体
pub struct TextSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadParam{
    name:    String,
    content: String,
    embed:   String,
}

// Gemini APIのレスポンス構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<EmbeddingData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

// エンベディング結果を格納する構造体
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
}


fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f64, Box<dyn std::error::Error>> {
    if a.len() != b.len() {
        return Err(Box::new(VectorLengthError));
    }

    let mut dot_product = 0.0_f64;
    let mut a_magnitude = 0.0_f64;
    let mut b_magnitude = 0.0_f64;

    for i in 0..a.len() {
        dot_product += (a[i] * b[i]) as f64;
        a_magnitude += (a[i] * a[i]) as f64;
        b_magnitude += (b[i] * b[i]) as f64;
    }

    if a_magnitude == 0.0 || b_magnitude == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (a_magnitude.sqrt() * b_magnitude.sqrt()))
}

async fn EmbedUserQuery(query :String) -> Vec<f32> {

    let items : Vec<f32> = Vec::new();
    let body = json!({
      "model": &MODEL_NAME.to_string(),
      "content": {"parts":[{"text": query.to_string()}]}
    });

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&GEMINI_API_KEY).unwrap());

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
        print_type_of(&embed_values[0]);
        println!("    エンベディング次元数: {}", embed_values.len());
        println!("    最初の5要素: {:?}", &embed_values[..embed_values.len().min(5)]);
        return embed_values;
    } else {
        println!("Error: {:?}", res.text().await.unwrap());
    }

   return items;
}

fn conver_u8_to_f32(data: Vec<u8>) -> Vec<f32>{
    let floats: &[f32] = cast_slice(&data);
    //println!("{:?}", floats);
    return floats.to_vec();
}

/**
*
* @param
*
* @return
*/
async fn CheckSimalirity(query: String) -> String {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct EmbedItem {
        name: String,
        content: String,
        embeddings: Vec<u8>
    }
    let input_f32 = EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());

    let con_str = POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");      
    
    let sql = "SELECT name, content, embeddings FROM embeddings".to_string();
    println!("sql={}", sql);

    let rows = sqlx::query(&sql)
        .fetch_all(&pool)
        .await.unwrap();
    let embed_items: Vec<EmbedItem> = rows
        .into_iter()
        .map(|row| EmbedItem {
            name: row.get("name"),
            content: row.get("content"),
            embeddings: row.get("embeddings"),
        })
        .collect();
        
    println!("emb.len={}", embed_items.len());
    let mut matches : String = "".to_string();
    //println!("emb.len[0]={}", embed_items[0].embeddings.len());
    //println!("    最初の5要素: {:?}", &embed_items[0].embeddings[..embed_items[0].embeddings.len().min(5)]);
    for v in &embed_items {
        let f32_value = conver_u8_to_f32(v.embeddings.clone());
        match cosine_similarity(&input_f32, &f32_value) {
            Ok(similarity) => {
                println!("cosine_similarity= {}", similarity);
                if similarity > 0.6 {
                    matches.push_str(&v.content.clone());
                }
            }
            Err(e) => eprintln!("エラー: {}", e),
        }
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

/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() {
    let query = "二十四節気".to_string();
    let input = CheckSimalirity(query).await;
    let send_text = format!("日本語で、回答して欲しい。\n{}", input);
    println!("send_text={}\n", send_text);

    // APII send
    let send_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent".to_string();

    let body = json!({
        "contents": [
        {
            "parts": [
            {
                "text": &send_text
            }
            ]
        }
        ]
    });
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&GEMINI_API_KEY).unwrap());
    let res = client
        .post(&send_url)
        .headers(headers)
        .json(&body)
        .send()
        .await.unwrap();

    println!("Status: {:?}", res.status());  
    if res.status().is_success() {
        let response_body: Value = res.json().await.unwrap();
        println!("response_body={}", response_body);

    } else {
        println!("Error: {:?}", res.text().await.unwrap());
    }    
}
/**
*
* @param
*
* @return
*/
fn f32_vec_to_u8_vec(data: &Vec<f32>) -> &[u8] {
    let len = data.len() * std::mem::size_of::<f32>();

    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            len,
        )
    }
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

use bytemuck::{cast_slice, Pod, Zeroable};
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

#[derive(Debug)]
struct VectorLengthError;

impl fmt::Display for VectorLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vectors must have the same length")
    }
}
impl std::error::Error for VectorLengthError {}

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
struct RagSearchParams {
    input_text: String,
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
      "model": &super::MODEL_NAME.to_string(),
      "content": {"parts":[{"text": query.to_string()}]}
    });

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&super::GEMINI_API_KEY).unwrap());

    let send_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent".to_string();
    
    // --- POST 送信 ---
    let res = client
        .post(&send_url)
        .headers(headers)
        .json(&body)
        .send()
        .await.unwrap();

    //println!("Status: {:?}", res.status());
    if res.status().is_success() {
        let response_body: Value = res.json().await.unwrap();
        
        // エンベディングデータを取得
        let embed_values = response_body["embedding"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();
        
        //print!("[0]=");
        //print_type_of(&embed_values[0]);
        //println!("    エンベディング次元数: {}", embed_values.len());
        //println!("    最初の5要素: {:?}", &embed_values[..embed_values.len().min(5)]);
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
    //println!("input_f32.len={}", input_f32.len());

    let con_str = super::POSTGRES_CONNECTION_STR.to_string();
    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");      
    
    let sql = "SELECT name, content, embeddings FROM embeddings".to_string();
    //println!("sql={}", sql);

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
        
    let mut matches : String = "".to_string();
    for v in &embed_items {
        let f32_value = conver_u8_to_f32(v.embeddings.clone());
        match cosine_similarity(&input_f32, &f32_value) {
            Ok(similarity) => {
                //println!("cosine_similarity= {}", similarity);
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
pub async fn rag_search_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
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
        match serde_json::from_value::<RagSearchParams>(arguments.clone()) {
          Ok(item_list_params) => {
            let input_text = item_list_params.input_text.clone();
            let input = CheckSimalirity(input_text).await;
            let send_text = format!("日本語で、回答して欲しい。\n{}", input);
            //println!("send_text={}\n", send_text);
            // API send
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
            headers.insert("x-goog-api-key", HeaderValue::from_str(&super::GEMINI_API_KEY).unwrap());
            let res = client
            .post(&send_url)
            .headers(headers)
            .json(&body)
            .send()
            .await.unwrap();

            println!("Status: {:?}", res.status());  
            let mut out_text: String = "".to_string();
            if res.status().is_success() {
                let response_body: Value = res.json().await.unwrap();
                //println!("response_body={}", response_body);
                out_text = serde_json::to_string(&response_body).unwrap();
            } else {
                out_text = "error, res.status=NG".to_string();
                println!("Error: {:?}", res.text().await.unwrap());
            } 
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

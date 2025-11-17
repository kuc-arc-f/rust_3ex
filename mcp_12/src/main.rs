use dotenvy::dotenv;
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;

use std::env;
use std::fs;
use std::path::Path;
use std::io::{self, Read};

use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

static MODEL_NAME: &str = "models/gemini-embedding-001";

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

impl TextSplitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        assert!(chunk_overlap < chunk_size, "オーバーラップはチャンクサイズより小さくする必要があります");
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    /// 基本的な文字ベース分割
    pub fn split_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<&str> = text.graphemes(true).collect();
        
        let mut start = 0;
        while start < chars.len() {
            let end = std::cmp::min(start + self.chunk_size, chars.len());
            let chunk: String = chars[start..end].iter().copied().collect();
            chunks.push(chunk);
            
            if end >= chars.len() {
                break;
            }
            start += self.chunk_size - self.chunk_overlap;
        }
        
        chunks
    }

    /// 再帰的分割（段落 -> 文 -> 単語の順）
    pub fn recursive_split(&self, text: &str) -> Vec<String> {
        let separators = vec!["\n\n", "\n", "。", ".", " "];
        self.recursive_split_with_separators(text, &separators)
    }

    fn recursive_split_with_separators(&self, text: &str, separators: &[&str]) -> Vec<String> {
        if separators.is_empty() {
            return self.split_text(text);
        }

        let separator = separators[0];
        let remaining_separators = &separators[1..];
        
        let parts: Vec<&str> = text.split(separator).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for part in parts {
            let part_len = part.graphemes(true).count();
            let current_len = current_chunk.graphemes(true).count();

            if current_len + part_len + separator.len() <= self.chunk_size {
                if !current_chunk.is_empty() {
                    current_chunk.push_str(separator);
                }
                current_chunk.push_str(part);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }

                if part_len > self.chunk_size {
                    // さらに細かく分割
                    let sub_chunks = self.recursive_split_with_separators(part, remaining_separators);
                    chunks.extend(sub_chunks);
                } else {
                    current_chunk = part.to_string();
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// センテンス単位での分割
    pub fn split_by_sentences(&self, text: &str) -> Vec<String> {
        let sentences: Vec<&str> = text
            .split(|c| c == '。' || c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in sentences {
            let sentence_with_period = format!("{}。", sentence.trim());
            let sentence_len = sentence_with_period.graphemes(true).count();
            let current_len = current_chunk.graphemes(true).count();

            if current_len + sentence_len <= self.chunk_size {
                current_chunk.push_str(&sentence_with_period);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                }
                current_chunk = sentence_with_period;
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

/**
*
* @param
*
* @return
*/
fn readTextData()-> anyhow::Result<Vec<ReadParam>, String> {
    let splitter = TextSplitter::new(500, 100);
    // 読み込み対象のフォルダパスを指定
    let folder_path = Path::new("./data/");
    let mut read_items: Vec<ReadParam> = Vec::new();
    let mut row_file_name :String= "".to_string();
    let mut row_file_cont :String = "".to_string();

    // フォルダが存在するか確認
    if !folder_path.is_dir() {
        // 存在しない場合は作成するか、処理を終了する
        eprintln!("エラー: フォルダ '{}' が存在しません。", folder_path.display());
        // 以下の行をコメントアウトしてフォルダ作成処理を追加しても良い
        // fs::create_dir_all(folder_path)?;
        return Err("error, folder none".to_string()); 
    }

    println!("--- フォルダ: {} 内の .txt ファイルを読み込みます ---", folder_path.display());

    // フォルダ内のエントリをイテレート
    for entry in fs::read_dir(folder_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // ファイルであり、拡張子が ".txt" であることを確認

        if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
            println!("\n[ファイル: {}]", path.display());
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(); // OsStrをStringに変換（エラーを無視）

            println!("filename={}", filename); 
            row_file_name = filename.to_string();
            
            // ファイルを開く
            match fs::File::open(&path) {
                Ok(mut file) => {
                    // ファイルの内容を保持するためのString
                    let mut contents = String::new();
                    
                    // ファイル全体を文字列に読み込む
                    match file.read_to_string(&mut contents) {
                        Ok(_) => {
                            // 読み込んだ内容を出力
                            println!("内容:\n{}", contents);
                            row_file_cont = contents.to_string();
                            println!("\n=== 再帰的分割 ===");
                            let recursive_chunks = splitter.recursive_split(&row_file_cont);
                            for (i, chunk) in recursive_chunks.iter().enumerate() {
                                println!("チャンク {}: {}", i + 1, chunk);
                                read_items.push(ReadParam{
                                    name: row_file_name.clone(),
                                    content: chunk.clone(),
                                    embed: "".to_string(),
                                })                                
                            }
                            /*
                            read_items.push(ReadParam{
                                name: row_file_name,
                                content: row_file_cont,
                                embed: "".to_string(),
                            })
                            */
                        },
                        Err(e) => {
                            eprintln!("エラー: ファイル '{}' の読み込み中にエラーが発生しました: {}", path.display(), e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("エラー: ファイル '{}' を開けませんでした: {}", path.display(), e);
                }
            }
        }
    }
    //println!("{:?}", read_items);
    println!("--- 読み込み完了 ---");
    return Ok(read_items);
}

// Gemini APIのレスポンス構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<EmbeddingData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

// エンベディング結果を格納する構造体
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub text: String,
    pub embedding: Vec<f32>,
}

/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let con_str = env::var("POSTGRES_CONNECTION_STR").expect("POSTGRES_CONNECTION_STR must be set");

    let pool = PgPoolOptions::new().max_connections(5)
    .connect(&con_str).await.expect("Failed to create pool");   

    let file_items = readTextData().unwrap();
    if file_items.len() == 0 {
        print!("error, file_items = 0");
        return;
    }
    //println!("{:?}", file_items[0]);

    let mut cont_Items: Vec<String> = Vec::new(); 
    
    for row_file in &file_items {
        println!("name={}", row_file.name);
        cont_Items.push(row_file.content.to_string());
    }
    
    println!("読み込んだテキスト数: {}", cont_Items.len());
    
    let requests: Vec<Value> = cont_Items
        .iter()
        .map(|text| {
            json!({
                "model": MODEL_NAME,
                "content": {
                    "parts": [{
                        "text": text
                    }]
                }
            })
        })
        .collect();

    let body = json!({
        "requests": requests
    });

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(&api_key).unwrap());

    let send_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:batchEmbedContents".to_string();
    
    // --- POST 送信 ---
    let res = client
        .post(&send_url)
        .headers(headers)
        .json(&body)
        .send()
        .await.unwrap();

    println!("Status: {:?}", res.status());
    
    if res.status().is_success() {
        let response_text = res.text().await.unwrap();
        
        // JSONレスポンスをパース
        match serde_json::from_str::<EmbeddingResponse>(&response_text) {
            Ok(embedding_response) => {
                // Vec構造体に格納
                let mut embedding_results: Vec<EmbeddingResult> = Vec::new();
                
                for (i, embedding_data) in embedding_response.embeddings.iter().enumerate() {
                    if i < cont_Items.len() {
                        embedding_results.push(EmbeddingResult {
                            text: cont_Items[i].clone(),
                            embedding: embedding_data.values.clone(),
                        });
                    }
                }
                
                // 結果を表示
                println!("\n=== エンベディング結果 ===");
                for (i, result) in embedding_results.iter().enumerate() {
                    //println!("\n[{}] テキスト: {}", i + 1, &result.text[..result.text.len().min(50)]);
                    println!("    エンベディング次元数: {}", result.embedding.len());
                    //println!("    最初の5要素: {:?}", &result.embedding[..result.embedding.len().min(5)]);
                    println!("    name: {:?}", &file_items[i].name);
                    let bytes_u8 = f32_vec_to_u8_vec(&result.embedding);
                    //println!("u8.len={}", bytes_u8.len());

                    let new_id = Uuid::new_v4();
                    let sessid = "".to_string();
                    let result = sqlx::query(
                        r#"
                        INSERT INTO embeddings (id, sessid, name, content, embeddings) 
                        VALUES ($1, $2, $3 , $4 , $5);
                        "# 
                    )
                    .bind(new_id)
                    .bind(&sessid)
                    .bind(&file_items[i].name.clone())
                    .bind(&result.text.clone())
                    .bind(&bytes_u8)
                    .execute(&pool)
                    .await.unwrap();                    
                }
                
                println!("\n総エンベディング数: {}", embedding_results.len());
            },
            Err(e) => {
                eprintln!("JSONパースエラー: {}", e);
                println!("レスポンス内容: {}", response_text);
            }
        }
    } else {
        let error_text = res.text().await.unwrap();
        eprintln!("APIエラー: {}", error_text);
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

use chrono::{Utc, DateTime};
use libsql::Database;
use libsql::Builder;
use libsql::Connection;
use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use umya_spreadsheet::*;

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
#[derive(Debug, Deserialize,Serialize)]
struct PurchaseDeleteParams {
    id: i32,
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

/**
*
* @param
*
* @return
*/
pub async fn purchase_list_excel_handler(
    params: Value, request_id: Option<Value>
) -> super::JsonRpcResponse {

    #[derive(Debug, Deserialize)]
    struct ListExcelParams {
        template_purchase: String,
        xls_out_dir: String,
    }    

    #[derive(Debug, Deserialize)]
    struct ItemData {
        name: String,
        price: i32,
    }    
    let mut template_name : String = "".to_string();
    let mut out_dir : String = "".to_string();
    if let Some(arguments) = params.get("arguments") {
        match serde_json::from_value::<ListExcelParams>(arguments.clone()) {
            Ok(purchase_params) => {
                template_name = purchase_params.template_purchase.clone();
                out_dir = purchase_params.xls_out_dir.clone();
            },
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
    println!("template_name={}", template_name);
    println!("out_dir={}", out_dir);

    // 1. 既存ファイルの読み込み
    //let path = Path::new("input.xlsx");
    let path = Path::new(&template_name);
    let mut book = reader::xlsx::read(path).unwrap();

    let url = super::TURSO_DATABASE_URL.to_string();
    let token = super::TURSO_AUTH_TOKEN.to_string();
    println!("TURSO_DATABASE_URL={}", url);
    let db = Builder::new_remote(url, token).build().await.unwrap();
    let conn = db.connect().unwrap();

    let order_sql = "ORDER BY created_at DESC LIMIT 10;";
    //let sql = format!("SELECT id, data , strftime('%Y-%m-%d', created_at) as created_at , updated_at 
    let sql = format!("SELECT id, data , created_at , updated_at 
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
    // 2. シート選択とセル編集
    let mut out_str: String = "".to_string();
    let mut time_msec : u128 = 0;
    match get_timestamp_milliseconds() {
        Ok(ms) => {
          println!("UNIXエポックからのミリ秒: {}", ms);
          time_msec = ms;
        },
        Err(e) => eprintln!("エラー: {:?}", e),
    }   
    let out_filename = format!("purchase_{}.xlsx", time_msec);   
    //let out_file_path = format!("{}/purchase_{}.xlsx", out_dir, time_msec);   
    let out_file_path = format!("{}/{}", out_dir, out_filename);   
    println!("out_filename={}", out_filename); 
    println!("out_file_path={}", out_file_path); 
    {
        let sheet = book
            .get_sheet_by_name_mut("Sheet1")
            .ok_or("Sheet1 が見つかりません").unwrap();
        let mut count = 2;
        let mut a_col_str : String;
        let mut b_col_str : String;
        let mut c_col_str : String;
        let mut a_val = "".to_string();
        let mut c_val = "".to_string();

        for item in &todos {
            let row_item: ItemData = serde_json::from_str(&item.data).expect("data JSON decord error");
            println!("デコードされた構造体: {:?}", row_item);          
            println!("ID: {}, Name: {}", item.id, item.created_at);
            // セル A1 に文字列
            a_col_str = format!("A{}", count);
            a_val = format!("{}", &item.id);
            sheet.get_cell_mut(a_col_str).set_value(a_val);
            b_col_str = format!("B{}", count);
            sheet.get_cell_mut(b_col_str).set_value(&row_item.name);
            c_col_str = format!("C{}", count);
            c_val = format!("{}", &row_item.price);
            sheet.get_cell_mut(c_col_str).set_value(c_val);
            // List-Data
            let row_str: String = format!("* id: {} , name= {} price= {}\n", &item.id, &row_item.name, &row_item.price);
            println!("row_str: {:?}", row_str); 
            out_str = format!("{}{}", &out_str, &row_str);            
            count = count + 1;
        }
        out_str = format!("{}{}", &out_str, "***\n* 下記リンクをおすと、ダウンロードできます。\n\n");     
        out_str = format!("{}[ Download Excel ](/data/{})\n", &out_str, out_filename);
    }

    // 5. 編集した内容で新ファイルに保存
    writer::xlsx::write(&book, Path::new(&out_file_path)).unwrap();
    return super::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: Some(json!({
            "content": [
                {
                    "type": "text",
                    "text": out_str.to_string()
                }
            ]
        })),
        error: None,
    };           
}

/**
*
* @param
*
* @return
*/
pub async fn purchase_delete_handler(params: Value, request_id: Option<Value>) -> super::JsonRpcResponse 
{
    if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
        if tool_name == "purchase_delete" {
            let url = super::TURSO_DATABASE_URL.to_string();
            let token = super::TURSO_AUTH_TOKEN.to_string();
            //println!("TURSO_DATABASE_URL={}", url);
            let db = Builder::new_remote(url, token).build().await.unwrap();
            let conn = db.connect().unwrap();    

            if let Some(arguments) = params.get("arguments") {
                match serde_json::from_value::<PurchaseDeleteParams>(arguments.clone()) {
                    Ok(purchase_params) => {
                        let id_value = purchase_params.id;
                        //select-id
                        let select_sql = format!("SELECT id, data ,created_at, updated_at 
                        FROM item_price
                        WHERE id= {} ;
                        "
                        , id_value
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

                        let sql = format!("DELETE FROM item_price WHERE id = {}", id_value);
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


fn get_timestamp_milliseconds() -> Result<u128, std::time::SystemTimeError> {
    let start = SystemTime::now();
    
    // UNIXエポックからの経過時間をDurationで取得
    let duration = start.duration_since(UNIX_EPOCH)?;
    
    // それをミリ秒単位の数値に変換
    let milliseconds = duration.as_millis();
    
    Ok(milliseconds)
}
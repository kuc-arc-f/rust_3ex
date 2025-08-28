use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use dotenvy::dotenv;

//static TARGET_DIR: &str = "/prog/Obsidian/vault/testvault";

fn get_ext_items() -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    items.push(".js".to_string());
    items.push(".jsx".to_string());
    items.push(".ts".to_string());
    items.push(".tsx".to_string());
    items.push(".json".to_string());
    items.push(".css".to_string());
    items.push(".htm".to_string());
    items.push(".html".to_string());
    items.push(".vue".to_string());
    items.push(".svelte".to_string());

    items.push(".py".to_string());
    items.push(".go".to_string());
    items.push(".rs".to_string());
    items.push(".php".to_string());

    return items;
}

fn list_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 再帰的に処理
                files.extend(list_files(&path)?);
            } else {
                // フルパスをVecに格納
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn main() -> std::io::Result<()> {
    dotenv().ok();

    let code_path = env::var("DATA_PATH")
      .expect("DATA_PATH must be set");
    println!("code_path: {}", code_path);

    let ext_items = get_ext_items();
    //println!("{:?}", ext_items);

    let target_dir = Path::new(&code_path); // ここを任意のフォルダに変更
    let files = list_files(target_dir)?;
    //println!("{:?}", files);
    for file in &files {
        println!("file: {}", file.display());
        let mut parent_dir = "";
        let mut stem_name = "";
        if let Some(parent) = file.parent() {
            let buf_parent_dir = parent.display();
            parent_dir = &buf_parent_dir.to_string();
            //println!("親ディレクトリのパス: {}", parent.display());
        }
        // ファイル名 (OsStr → &str に変換)
        //if let Some(file_name) = file.file_name() {
        //    println!("ファイル名: {}", file_name.to_string_lossy());
        //}        
        // 拡張子を除いたファイル名 (stem)
        if let Some(stem) = file.file_stem() {
            stem_name = &stem.to_string_lossy();
            //println!("拡張子なしファイル名: {}", stem.to_string_lossy());
        }
        // 拡張子
        if let Some(ext) = file.extension() {
            let ext_tmp = ext.to_string_lossy();
            let ext_dot = format!("{}{}", ".", ext_tmp);
            //println!("ext_dot: {}", ext_dot);
            let vExt = ext_items.iter().any(|s| s == &ext_dot);
            println!("vExt: {}", vExt);
            if ext != "md" && vExt {
                let old_path = Path::new(file);
                println!("old_path: {}", old_path.display());
                let new_path = format!("{}{}", old_path.display(), ".md"); 
                println!("new_path: {}", &new_path);
                fs::rename(&old_path, &new_path)?;
            }
        }
    }
    Ok(())
}
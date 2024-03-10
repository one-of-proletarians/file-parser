#[macro_use]
extern crate dotenv_codegen;

mod parser_v2;
use parser_v2::parse;

use std::{fs::OpenOptions, io::Write, path::Path};

fn main() {
    let path = Path::new("B1-K1.txt");
    let result_path = Path::new("result.json");

    let fields = match parse(path) {
        Ok(x) => x,
        Err(_) => {
            println!("ошибка открытия файла");
            return;
        }
    };

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(result_path)
        .expect("Error opening")
        .write(serde_json::to_string_pretty(&fields).unwrap().as_bytes())
        .unwrap();
}

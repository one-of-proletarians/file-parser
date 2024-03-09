#[macro_use]
extern crate dotenv_codegen;

mod parser;
use parser::{parse, to_json};

use std::{fs::OpenOptions, io::Write, path::Path};

fn main() {
    let file_path = Path::new("B1-K1.txt");
    let result_path = Path::new("result.json");

    let fields = parse(file_path, "DE", "RU");

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(result_path)
        .expect("Error opening")
        .write(to_json(&fields).as_bytes())
        .unwrap();
}

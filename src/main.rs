mod json;

use crate::json::*;

fn main() {
    let path: String = String::from("example.json");
    match parse_json(&path) {
        Some(res) => println!("Parsed JSON:\n{:#?}", res),
        None => println!("Failed to read JSON at: {}", path),
    }
}

fn main() {
    let path: &str = "example.json";
    match flambrew_json_rs::json::parse_json(&path) {
        Some(res) => println!("Parsed JSON:\n{:#?}", res),
        None => println!("Failed to read JSON at: {}", path),
    }
}

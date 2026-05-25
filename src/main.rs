use flambrew_json_rs::*;

fn main() {
    let path: &str = "example.json";
    match parse_json(path) {
        Ok(res) => println!("Parsed JSON:\n{:#?}", res),
        Err(err) => match err {
            JErr::Io(msg) => println!("{}", msg),
            JErr::Parse(msg) => println!("{}", msg),
        },
    }
}

use std::fs;

#[derive(Debug)]
pub struct NVPair {
    key: String,
    value: Value,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Value {
    Obj(Vec<NVPair>),
    Arr(Vec<Value>),
    Str(String),
    Bool(bool),
    Int(i64),
    Flt(f64),
    Null,
}

fn parse_string(json: &[u8], offset: &mut usize) -> String {
    *offset += 1;
    let mut out: String = String::new();
    loop { match json[*offset] {
        x if x == b'"' => {
            *offset += 1;
            return out;
        },
        _ => {
            out.push(json[*offset] as char);
            *offset += 1;
        },
    }}
}

fn parse_word(json: &[u8], offset: &mut usize, target: &'static str) -> bool {
    let i: usize = 0;
    loop { 
        let c: u8 = target.as_bytes()[i];
        match c {
            x if x == json[*offset] => *offset += 1,
            _ => return c == b'\0',
        }
    }
}

enum ArrayPhase {
    END,
    WS0,
    VAL,
    WS1,
}

fn parse_array(json: &[u8], offset: &mut usize) -> Option<Vec<Value>> {
    *offset += 1;
    let mut out: Vec<Value> = Vec::new();
    let mut phase: ArrayPhase = ArrayPhase::END;
    loop { match phase {
        ArrayPhase::END => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b']' => {
                *offset += 1;
                return Some(out)
            },
            _ => phase = ArrayPhase::WS0,
        },
        ArrayPhase::WS0 => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            _ => phase = ArrayPhase::VAL,
        },
        ArrayPhase::VAL => match parse_value(json, offset) {
            Some(val) => {
                out.push(val);
                phase = ArrayPhase::WS1;
            },
            None => return None,
        },
        ArrayPhase::WS1 => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b',' => { 
                phase = ArrayPhase::WS0;
                *offset += 1;
            },
            x if x == b']' => {
                *offset += 1;
                return Some(out)
            },
            _ => return None,
        },
    }}
}

enum NumberPhase {
    NEG,
    DIG,
    FRC,
    EXP,
}

fn parse_num(json: &[u8], offset: &mut usize) -> Option<Value> {
    let mut int: i64 = 0;
    // let mut flt: f64 = 0.;
    let mut phase: NumberPhase = NumberPhase::NEG;
    let mut negflag: i64 = 1;
    loop { match phase {
        NumberPhase::NEG => match json[*offset] {
            x if x == b'-' => { 
                negflag = -1;
                *offset += 1;
            },
            _ => phase = NumberPhase::DIG,
        },
        NumberPhase::DIG => match json[*offset] {
            x if x.is_ascii_digit() => {
                int = int * 10 + (x - b'0') as i64;
                *offset += 1;
            },
            x if x == b'.' => {
                // flt = int as f64;
                phase = NumberPhase::FRC;
                *offset += 1;
            },
            x if x == b'e' || x == b'E' => {
                // flt = int as f64;
                phase = NumberPhase::EXP;
                *offset += 1;
            },
            _ => return Some(Value::Int(int * negflag)),
        },
        NumberPhase::FRC => return None,
        NumberPhase::EXP => return None,
    }}
}

fn parse_value(json: &[u8], offset: &mut usize) -> Option<Value> {
    loop { match json[*offset] {
        x if x.is_ascii_whitespace() => *offset += 1,
        x if x == b'{' => 
            return match parse_object(json, offset) {
                Some(obj) => Some(Value::Obj(obj)),
                None => return None,
            },
        x if x == b'[' => 
            return match parse_array(json, offset) {
                Some(arr) => Some(Value::Arr(arr)),
                None => return None,
            },
        x if x == b'"' => return Some(Value::Str(parse_string(json, offset))),
        x if x == b't' => return Some(Value::Bool(parse_word(json, offset, "true"))),
        x if x == b'f' => return Some(Value::Bool(!parse_word(json, offset, "false"))),
        x if x == b'n' => 
            return if parse_word(json, offset, "null") { 
                Some(Value::Null) 
            } else { 
                None 
            },
        _ => return parse_num(json, offset),
    }} 
}

enum ObjectPhase {
    END,
    WS0,
    STR,
    WS1,
    VAL,
    WS2,
}

fn parse_object(json: &[u8], offset: &mut usize) -> Option<Vec<NVPair>> {
    *offset += 1;
    let mut out: Vec<NVPair> = Vec::new();
    let mut phase: ObjectPhase = ObjectPhase::END;
    loop { match phase {
        ObjectPhase::END => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b'}' => {
                *offset += 1;
                return Some(out)
            },
            _ => phase = ObjectPhase::WS0,
        },
        ObjectPhase::WS0 => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b'"' => {
                out.push(NVPair{ key: String::new(), value: Value::Null });
                phase = ObjectPhase::STR;
            },
            _ => return None,
        },
        ObjectPhase::STR => match parse_string(json, offset) {
            x if !x.is_empty() => {
                out.last_mut().unwrap().key = x;
                phase = ObjectPhase::WS1;
            },
            _ => return None,
        },
        ObjectPhase::WS1 => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b':' => {
                phase = ObjectPhase::VAL;
                *offset += 1;
            },
            _ => return None,
        },
        ObjectPhase::VAL => match parse_value(json, offset) {
            Some(val) => {
                out.last_mut().unwrap().value = val;
                phase = ObjectPhase::WS2;
            },
            None => return None,
        },
        ObjectPhase::WS2 => match json[*offset] {
            x if x.is_ascii_whitespace() => *offset += 1,
            x if x == b',' => { 
                phase = ObjectPhase::WS0;
                *offset += 1;
            },
            x if x == b'}' => {
                *offset += 1;
                return Some(out)
            },
            _ => return None,
        },
    }}
}

pub fn parse_json(path: &String) -> Option<Vec<NVPair>> {
    match fs::read_to_string(path) {
        Ok(json) => {
            let json = json.as_bytes();
            let mut offset: usize = 0;
            let mut out: Option<Vec<NVPair>> = None;
            while offset != json.len() {
                match json[offset] {
                    x if x == b'{' => out = Some(parse_object(json, &mut offset)?),
                    x if x.is_ascii_whitespace() => offset += 1,
                    _ => return None
                }
            }
            out
        },
        Err(_) => None,
    }  
}

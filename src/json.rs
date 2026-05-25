use std::fs;

#[derive(Debug)]
#[allow(dead_code)]
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

fn parse_word(json: &[u8], offset: &mut usize, target: &'static str) -> bool {
    for i in 0..target.len() {
        match target.as_bytes()[i] {
            c if c == json[*offset] => *offset += 1,
            _ => return false,
        }
    }
    true
}

fn parse_string(json: &[u8], offset: &mut usize) -> String {
    *offset += 1;
    let mut out: String = String::new();
    loop { match json[*offset] {
        b'"' => {
            *offset += 1;
            return out;
        },
        b'\\' => {
            *offset += 1;
            match json[*offset] {
               b'"' => out.push('"'),
               b'\\' => out.push('\\'),
               b'n' => out.push('\n'),
               b'r' => out.push('\r'),
               b't' => out.push('\t'),
               _ => {},
            }
            *offset += 1;
        },
        _ => {
            out.push(json[*offset] as char);
            *offset += 1;
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
    let mut flt: f64 = 0.;
    let mut dec: u32 = 1;
    let mut exp: f64 = 0.;
    let mut negflag: i64 = 1;
    let mut zeroflag: bool = false;
    let mut expsignflag: f64 = 0.;
    let mut phase: NumberPhase = NumberPhase::NEG;
    loop { match phase {
        NumberPhase::NEG => match json[*offset] {
            c if c == b'-' && negflag == 1 => { 
                negflag = -1;
            },
            _ => { 
                phase = NumberPhase::DIG; 
                continue; 
            },
        },
        NumberPhase::DIG => match json[*offset] {
            c if c == b'0' && int == 0 => {
                zeroflag = true;
            },
            c if c.is_ascii_digit() && !zeroflag => {
                int = int * 10 + (c - b'0') as i64;
            },
            b'.' => {
                flt = int as f64;
                phase = NumberPhase::FRC;
            },
            c if c == b'e' || c == b'E' => {
                flt = int as f64;
                phase = NumberPhase::EXP;
            },
            _ => break,
        },
        NumberPhase::FRC => match json[*offset] {
            c if c.is_ascii_digit() => {
                flt += (c - b'0') as f64 / i32::pow(10, dec) as f64;
                dec += 1;
            },
            c if c == b'e' || c == b'E' => {
                phase = NumberPhase::EXP;
            },
            _ => return Some(Value::Flt(flt * negflag as f64)),
        },
        NumberPhase::EXP => match json[*offset] {
            c if c == b'-' && expsignflag == 0. => expsignflag = -1.,
            c if c == b'+' && expsignflag == 0. => expsignflag = 1.,
            c if c.is_ascii_digit() => {
                if expsignflag == 0. {
                    expsignflag = 1.;
                }
                exp = exp * 10. + (c - b'0') as f64;
            },
            _ => return Some(Value::Flt(flt * ((10) as f64).powf(exp * expsignflag) * negflag as f64)),
        },
    } *offset += 1; }
    
    if int > 0 || zeroflag {
        return Some(Value::Int(int * negflag))
    } else {
        return None
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
            c if c.is_ascii_whitespace() => *offset += 1,
            b']' => {
                *offset += 1;
                return Some(out)
            },
            _ => phase = ArrayPhase::WS0,
        },
        ArrayPhase::WS0 => match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
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
            c if c.is_ascii_whitespace() => *offset += 1,
            b',' => { 
                phase = ArrayPhase::WS0;
                *offset += 1;
            },
            b']' => {
                *offset += 1;
                return Some(out)
            },
            _ => return None,
        },
    }}
}

fn parse_value(json: &[u8], offset: &mut usize) -> Option<Value> {
    loop { match json[*offset] {
        c if c.is_ascii_whitespace() => *offset += 1,
        b'{' => 
            return match parse_object(json, offset) {
                Some(obj) => Some(Value::Obj(obj)),
                None => return None,
            },
        b'[' => 
            return match parse_array(json, offset) {
                Some(arr) => Some(Value::Arr(arr)),
                None => return None,
            },
        b'"' => return Some(Value::Str(parse_string(json, offset))),
        b't' => return Some(Value::Bool(parse_word(json, offset, "true"))),
        b'f' => return Some(Value::Bool(!parse_word(json, offset, "false"))),
        b'n' => 
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
    let mut name: String = String::new();
    let mut out: Vec<NVPair> = Vec::new();
    let mut phase: ObjectPhase = ObjectPhase::END;
    loop { match phase {
        ObjectPhase::END => match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
            b'}' => {
                *offset += 1;
                return Some(out)
            },
            _ => phase = ObjectPhase::WS0,
        },
        ObjectPhase::WS0 => match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
            b'"' => {
                phase = ObjectPhase::STR;
            },
            _ => return None,
        },
        ObjectPhase::STR => match parse_string(json, offset) {
            x if !x.is_empty() => {
                name = x;
                phase = ObjectPhase::WS1;
            },
            _ => return None,
        },
        ObjectPhase::WS1 => match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
            b':' => {
                phase = ObjectPhase::VAL;
                *offset += 1;
            },
            _ => return None,
        },
        ObjectPhase::VAL => match parse_value(json, offset) {
            Some(val) => {
                out.push(NVPair{ key: name.clone(), value: val });
                phase = ObjectPhase::WS2;
            },
            None => return None,
        },
        ObjectPhase::WS2 => match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
            b',' => { 
                phase = ObjectPhase::WS0;
                *offset += 1;
            },
            b'}' => {
                *offset += 1;
                return Some(out)
            },
            _ => return None,
        },
    }}
}

pub fn parse_json(path: &str) -> Option<Value> {
    match fs::read_to_string(path) {
        Ok(json) => {
            let json = json.as_bytes();
            let mut offset: usize = 0;
            let mut parsed: bool = false;
            let mut out: Option<Value> = None;
            while offset < json.len() { match json[offset] {
                c if c.is_ascii_whitespace() => offset += 1,
                _ if !parsed => {
                    out = Some(parse_value(json, &mut offset)?);
                    parsed = true;
                },
                _ => return None,
            }}

            out
        },
        Err(_) => None,
    }  
}

use std::fs;

const ERR_PARSE_FAIL: &str = "Failed to parse JSON";

#[derive(Debug)]
#[allow(dead_code)]
pub enum JErr {
    Io(std::io::Error),
    Parse(&'static str),
}

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
    loop {
        match json[*offset] {
            b'"' => {
                *offset += 1;
                return out;
            }
            b'\\' => {
                *offset += 1;
                match json[*offset] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    _ => {}
                }
                *offset += 1;
            }
            _ => {
                out.push(json[*offset] as char);
                *offset += 1;
            }
        }
    }
}

enum NumberPhase {
    Neg,
    Dig,
    Fre,
    Exp,
}

fn parse_num(json: &[u8], offset: &mut usize) -> Option<Value> {
    let mut int: i64 = 0;
    let mut flt: f64 = 0.;
    let mut dec: u32 = 1;
    let mut exp: f64 = 0.;
    let mut negflag: i64 = 1;
    let mut zeroflag: bool = false;
    let mut expsignflag: f64 = 0.;
    let mut phase: NumberPhase = NumberPhase::Neg;
    loop {
        match phase {
            NumberPhase::Neg => match json[*offset] {
                c if c == b'-' && negflag == 1 => {
                    negflag = -1;
                }
                _ => {
                    phase = NumberPhase::Dig;
                    continue;
                }
            },
            NumberPhase::Dig => match json[*offset] {
                c if c == b'0' && int == 0 => {
                    zeroflag = true;
                }
                c if c.is_ascii_digit() && !zeroflag => {
                    int = int * 10 + (c - b'0') as i64;
                }
                b'.' => {
                    flt = int as f64;
                    phase = NumberPhase::Fre;
                }
                c if c == b'e' || c == b'E' => {
                    flt = int as f64;
                    phase = NumberPhase::Exp;
                }
                _ => break,
            },
            NumberPhase::Fre => match json[*offset] {
                c if c.is_ascii_digit() => {
                    flt += (c - b'0') as f64 / i32::pow(10, dec) as f64;
                    dec += 1;
                }
                c if c == b'e' || c == b'E' => {
                    phase = NumberPhase::Exp;
                }
                _ => return Some(Value::Flt(flt * negflag as f64)),
            },
            NumberPhase::Exp => match json[*offset] {
                c if c == b'-' && expsignflag == 0. => expsignflag = -1.,
                c if c == b'+' && expsignflag == 0. => expsignflag = 1.,
                c if c.is_ascii_digit() => {
                    if expsignflag == 0. {
                        expsignflag = 1.;
                    }
                    exp = exp * 10. + (c - b'0') as f64;
                }
                _ => {
                    return Some(Value::Flt(
                        flt * ((10) as f64).powf(exp * expsignflag) * negflag as f64,
                    ));
                }
            },
        }
        *offset += 1;
    }

    if int > 0 || zeroflag {
        Some(Value::Int(int * negflag))
    } else {
        None
    }
}

enum ArrayPhase {
    End,
    Ws0,
    Val,
    Ws1,
}

fn parse_array(json: &[u8], offset: &mut usize) -> Option<Vec<Value>> {
    *offset += 1;
    let mut out: Vec<Value> = Vec::new();
    let mut phase: ArrayPhase = ArrayPhase::End;
    loop {
        match phase {
            ArrayPhase::End => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b']' => {
                    *offset += 1;
                    return Some(out);
                }
                _ => phase = ArrayPhase::Ws0,
            },
            ArrayPhase::Ws0 => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                _ => phase = ArrayPhase::Val,
            },
            ArrayPhase::Val => match parse_value(json, offset) {
                Some(val) => {
                    out.push(val);
                    phase = ArrayPhase::Ws1;
                }
                None => return None,
            },
            ArrayPhase::Ws1 => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b',' => {
                    phase = ArrayPhase::Ws0;
                    *offset += 1;
                }
                b']' => {
                    *offset += 1;
                    return Some(out);
                }
                _ => return None,
            },
        }
    }
}

fn parse_value(json: &[u8], offset: &mut usize) -> Option<Value> {
    loop {
        match json[*offset] {
            c if c.is_ascii_whitespace() => *offset += 1,
            b'{' => {
                return match parse_object(json, offset) {
                    Some(obj) => Some(Value::Obj(obj)),
                    None => return None,
                };
            }
            b'[' => {
                return match parse_array(json, offset) {
                    Some(arr) => Some(Value::Arr(arr)),
                    None => return None,
                };
            }
            b'"' => return Some(Value::Str(parse_string(json, offset))),
            b't' => return Some(Value::Bool(parse_word(json, offset, "true"))),
            b'f' => return Some(Value::Bool(!parse_word(json, offset, "false"))),
            b'n' => {
                return if parse_word(json, offset, "null") {
                    Some(Value::Null)
                } else {
                    None
                };
            }
            _ => return parse_num(json, offset),
        }
    }
}

enum ObjectPhase {
    End,
    Ws0,
    Str,
    Ws1,
    Val,
    Ws2,
}

fn parse_object(json: &[u8], offset: &mut usize) -> Option<Vec<NVPair>> {
    *offset += 1;
    let mut name: String = String::new();
    let mut out: Vec<NVPair> = Vec::new();
    let mut phase: ObjectPhase = ObjectPhase::End;
    loop {
        match phase {
            ObjectPhase::End => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b'}' => {
                    *offset += 1;
                    return Some(out);
                }
                _ => phase = ObjectPhase::Ws0,
            },
            ObjectPhase::Ws0 => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b'"' => {
                    phase = ObjectPhase::Str;
                }
                _ => return None,
            },
            ObjectPhase::Str => match parse_string(json, offset) {
                x if !x.is_empty() => {
                    name = x;
                    phase = ObjectPhase::Ws1;
                }
                _ => return None,
            },
            ObjectPhase::Ws1 => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b':' => {
                    phase = ObjectPhase::Val;
                    *offset += 1;
                }
                _ => return None,
            },
            ObjectPhase::Val => match parse_value(json, offset) {
                Some(val) => {
                    out.push(NVPair {
                        key: name.clone(),
                        value: val,
                    });
                    phase = ObjectPhase::Ws2;
                }
                None => return None,
            },
            ObjectPhase::Ws2 => match json[*offset] {
                c if c.is_ascii_whitespace() => *offset += 1,
                b',' => {
                    phase = ObjectPhase::Ws0;
                    *offset += 1;
                }
                b'}' => {
                    *offset += 1;
                    return Some(out);
                }
                _ => return None,
            },
        }
    }
}

pub fn parse_json(path: &str) -> Result<Value, JErr> {
    let json = fs::read_to_string(path).map_err(JErr::Io)?;
    let bytes = json.as_bytes();

    let mut offset: usize = 0;
    let mut parsed: bool = false;
    let mut out: Result<Value, JErr> = Err(JErr::Parse(ERR_PARSE_FAIL));
    while offset < bytes.len() {
        match bytes[offset] {
            c if c.is_ascii_whitespace() => offset += 1,
            _ => {
                out = if parsed {
                    return Err(JErr::Parse(ERR_PARSE_FAIL));
                } else {
                    match parse_value(bytes, &mut offset) {
                        Some(obj) => {
                            parsed = true;
                            Ok(obj)
                        }
                        None => return Err(JErr::Parse(ERR_PARSE_FAIL)),
                    }
                }
            }
        }
    }

    out
}

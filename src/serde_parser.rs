extern crate heck;
extern crate nom;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use self::heck::CamelCase;
use self::serde_json::Value;
use nom::IResult;

use find_element;

fn json_val_to_rust(input: &Value) -> String {
    match input {
        &Value::String(ref s) => match s.as_ref() {
            "uint64" => "u64".to_string(),
            "uint32" => "u32".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i64".to_string(),
            "str" => "String".to_string(),
            _ => "String".to_string(),
        },
        &Value::Number(_) => "f64".to_string(),
        &Value::Bool(_) => "bool".to_string(),
        &Value::Null => "None".to_string(),
        &Value::Object(_) => "struct".to_string(),
        &Value::Array(_) => "Vec<String>".to_string(),
    }
}

// Prefix reserved words with qemu__
// Returns Some(String) if it changed it, None otherwise
fn reserved_words(input: &String) -> Option<String> {
    if input.starts_with("type") || input.starts_with("in") || input.starts_with("static")
        || input.starts_with("abstract")
    {
        return Some(format!("qemu_{}", input));
    }

    None
}

fn print_struct(v: serde_json::Map<String, Value>) -> Result<String, String> {
    let mut output = String::new();
    let name = v.get("struct").unwrap().as_str().unwrap();
    if name == "String" {
        // Skip this weird wrapper thing
        return Ok("".into());
    }
    output.push_str("#[derive(Debug, Deserialize, Serialize)]");
    output.push_str("#[serde(rename_all = \"kebab-case\")]");
    output.push_str(&format!("pub struct {} {{", name));
    for (field_name, field_type) in v.get("data").unwrap().as_object().unwrap() {
        let n = field_name.replace("*", "").replace("-", "_");
        match reserved_words(&n) {
            Some(renamed) => {
                output.push_str(&format!(
                    "#[serde(rename = \"{}\")]\n\tpub {}: {},",
                    &n,
                    renamed,
                    json_val_to_rust(field_type)
                ));
            }
            None => {
                output.push_str(&format!("\tpub {}: {},", &n, json_val_to_rust(field_type)));
            }
        }
    }
    output.push_str("}}");
    Ok(output)
}

fn print_union(v: serde_json::Map<String, Value>) -> Result<String, String> {
    let mut output = String::new();
    let name = v.get("union").unwrap().as_str().unwrap();
    output.push_str("#[derive(Debug, Deserialize, Serialize)]");
    output.push_str(&format!("pub enum {} {{", name));
    for (field_name, field_type) in v.get("data").unwrap().as_object().unwrap() {
        let n = field_name.replace("*", "").replace("-", "_");
        match reserved_words(&n) {
            Some(renamed) => {
                output.push_str(&format!(
                    "#[serde(rename = \"{}\")]\n\t{}({}),",
                    &n,
                    renamed,
                    json_val_to_rust(field_type)
                ));
            }
            None => {
                output.push_str(&format!("\t{}({}),", &n, json_val_to_rust(field_type)));
            }
        }
    }
    output.push_str("}}");

    Ok(output)
}

fn print_command(v: serde_json::Map<String, Value>) -> Result<String, String> {
    let mut output = String::new();
    // { 'command': 'add_client',
    // 'data': { 'protocol': 'str', 'fdname': 'str', '*skipauth': 'bool',
    //            '*tls': 'bool' } }
    let name = v.get("command").unwrap().as_str().unwrap();
    // args are optional
    let data = v.get("data");
    // return type is optional, can be an array or just a plain type
    let return_type = v.get("returns");
    let mut fn_args: Vec<String> = Vec::new();

    let mut fn_definition = format!("pub fn {}_cmd(", name.replace("-", "_"));
    match data {
        Some(v) => match v {
            &Value::Object(ref o) => {
                let args: Vec<String> = o.iter()
                    .map(|(name, val)| {
                        let n = name.replace("*", "").replace("-", "_");
                        match reserved_words(&n) {
                            Some(renamed) => format!("{}: {}", renamed, json_val_to_rust(val)),
                            None => format!("{}: {}", &n, json_val_to_rust(val)),
                        }
                    })
                    .collect();
                fn_definition.push_str(&args.join(","));
                fn_args = o.keys()
                    .map(|s| {
                        let n = s.replace("*", "").replace("-", "_");
                        match reserved_words(&n) {
                            Some(renamed) => renamed,
                            None => n,
                        }
                    })
                    .collect();
            }
            _ => {
                return Err(format!("Unknown data field type: {:?}", data));
            }
        },
        None => {}
    };
    fn_definition.push_str(")");

    match return_type {
        Some(r) => match r {
            &Value::Array(ref a) => {
                fn_definition.push_str(&format!(
                    "->Result<Vec<{}>, String>",
                    a[0].as_str().unwrap()
                ));
            }
            &Value::String(ref s) => {
                let s = if s == "str" {
                    "String"
                } else if s == "int" {
                    "i64"
                } else {
                    s
                };
                fn_definition.push_str(&format!("->Result<{}, String>", s));
            }
            _ => {
                return Err(format!("Unknown return field type: {:?}", return_type));
            }
        },
        None => {
            fn_definition.push_str("->Result<(), String>");
        }
    };
    fn_definition.push_str("{");

    output.push_str(&format!("{}", fn_definition));
    output.push_str("let cmd = json!({{");
    output.push_str(&format!("\"execute\": \"{}\"", name));
    let json_args: Vec<String> = fn_args
        .iter()
        .map(|arg| format!("\"{}\": {}", arg, arg))
        .collect();
    if !json_args.is_empty() {
        output.push_str(",");
        output.push_str("\"arguments\": {{");
        output.push_str(&format!("{}", json_args.join(",")));
        output.push_str("}}");
    }
    output.push_str("}});");
    output.push_str("let ret = call_qemu(cmd)?;");
    if return_type.is_none() {
        output.push_str("Ok(())");
    } else {
        output.push_str("Ok(ret)");
    }
    output.push_str("}}");

    Ok(output)
}

fn print_enum(v: serde_json::Map<String, Value>) -> Result<String, String> {
    let mut output = String::new();
    //enum: {"data": Array([String("read"), String("write")]), "enum": String("IoOperationType")}
    let name = v.get("enum").unwrap().as_str().unwrap();
    output.push_str("#[derive(Debug, Deserialize, Serialize)]");
    output.push_str("#[serde(rename_all = \"kebab-case\")]");
    output.push_str(&format!("pub enum {} {{", name));
    // Enum can either contain an array of values or just a single value
    match v.get("data").unwrap() {
        &Value::Array(ref a) => for field in a {
            let f = field.as_str().unwrap();
            //let numbers = vec!["1", "2", "3"];
            if super::REPLACEMAP.contains_key(f) {
                //change to One, Two, Etc
                let replaced = super::REPLACEMAP.get(f).unwrap().clone();
                output.push_str(&format!(
                    "\t#[serde(rename = \"{}\")]\n\t{},",
                    f,
                    replaced.to_camel_case()
                ));
            } else {
                output.push_str(&format!("\t{},", f.to_camel_case()));
            }
        },
        _ => {
            return Err(format!("Unknown enum field: {:?}", v.get("data")));
        }
    };
    output.push_str("}}");

    Ok(output)
}

#[test]
fn test_generate_definitions() {
    let url = "https://raw.githubusercontent.com/elmarco/qemu/qapi/qapi-schema.json";
    generate_rust_definitions(&url);
}

pub fn generate_rust_definitions(url: &str) -> Result<String, String> {
    let definitions = get_definitions(url)?;
    let mut output = String::new();
    output.push_str("use call_qemu;");
    for d in definitions {
        match d {
            Value::Object(map) => {
                // Most things should be objects
                if map.contains_key("struct") {
                    print_struct(map)?;
                } else if map.contains_key("command") {
                    print_command(map)?;
                } else if map.contains_key("enum") {
                    print_enum(map)?;
                } else if map.contains_key("union") {
                    print_union(map)?;
                }
            }
            _ => {
                // Other value detected
                return Err(format!("Unknown value: {:?}", d));
            }
        }
    }
    Ok(output)
}

fn get_definitions(url: &str) -> Result<Vec<Value>, String> {
    let mut buff = String::new();
    let mut definitions: Vec<Value> = Vec::new();
    let text = reqwest::get(url)
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    for line in text.lines() {
        if !line.starts_with("#") {
            // For lines that have a comment in the middle of the line
            // we remove the # until \n
            if line.contains("#") {
                let v: Vec<&str> = line.split("#").collect();
                buff.push_str(v[0].into());
            } else {
                buff.push_str(line);
            }
        }
    }
    let mut elements: Vec<String> = Vec::new();
    let mut leftover = buff.as_bytes();

    // Try to extract all the definitions from the json text
    loop {
        match find_element(leftover) {
            IResult::Done(left, s) => {
                if s.is_empty() {
                    break;
                }
                elements.push(s);
                leftover = left;
            }
            _ => {
                break;
            }
        };
    }

    for e in elements {
        let s = e.replace("'", "\"");
        let v: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        definitions.push(v);
    }

    Ok(definitions)
}

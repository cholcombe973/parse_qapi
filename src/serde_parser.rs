extern crate heck;
extern crate nom;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use self::heck::CamelCase;
use self::serde_json::Result as SerdeResult;
use self::serde_json::Value;
use nom::IResult;

use {find_element, QemuType};

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
        &Value::Number(ref num) => "f64".to_string(),
        &Value::Bool(b) => "bool".to_string(),
        &Value::Null => "None".to_string(),
        &Value::Object(ref map) => "struct".to_string(),
        &Value::Array(ref values) => "Vec<String>".to_string(),
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

fn print_struct(v: serde_json::Map<String, Value>) -> Result<(), String> {
    let name = v.get("struct").unwrap().as_str().unwrap();
    if name == "String" {
        // Skip this weird wrapper thing
        return Ok(());
    }
    println!("#[derive(Debug, Deserialize, Serialize)]");
    println!("#[serde(rename_all = \"kebab-case\")]");
    println!("pub struct {} {{", name);
    for (field_name, field_type) in v.get("data").unwrap().as_object().unwrap() {
        let n = field_name.replace("*", "").replace("-", "_");
        match reserved_words(&n) {
            Some(renamed) => {
                println!(
                    "#[serde(rename = \"{}\")]\n\tpub {}: {},",
                    &n,
                    renamed,
                    json_val_to_rust(field_type)
                );
            }
            None => {
                println!("\tpub {}: {},", &n, json_val_to_rust(field_type));
            }
        }
    }
    println!("}}");
    Ok(())
}

fn print_command(v: serde_json::Map<String, Value>) -> Result<(), String> {
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
            fn_definition.push_str("->Result<Foo, String>");
        }
    };
    fn_definition.push_str("{");

    println!("{}", fn_definition);
    println!("let cmd = json!({{");
    println!("\"execute\": \"{}\"", name);
    let json_args: Vec<String> = fn_args
        .iter()
        .map(|arg| format!("\"{}\": {}", arg, arg))
        .collect();
    if !json_args.is_empty() {
        println!(",");
        println!("\"arguments\": {{");
        println!("{}", json_args.join(","));
        println!("}}");
    }
    println!("}});");
    println!("let ret = call_qemu(cmd)?;");
    println!("Ok(ret)");
    println!("}}");
    Ok(())
}

fn print_enum(v: serde_json::Map<String, Value>) -> Result<(), String> {
    //enum: {"data": Array([String("read"), String("write")]), "enum": String("IoOperationType")}
    let name = v.get("enum").unwrap().as_str().unwrap();
    println!("#[derive(Debug, Deserialize, Serialize)]");
    println!("#[serde(rename_all = \"kebab-case\")]");
    println!("pub enum {} {{", name);
    // Enum can either contain an array of values or just a single value
    match v.get("data").unwrap() {
        &Value::Array(ref a) => for field in a {
            println!("\t{},", field.as_str().unwrap().to_camel_case());
        },
        _ => {
            return Err(format!("Unknown enum field: {:?}", v.get("data")));
        }
    };
    println!("}}");

    Ok(())
}

#[test]
fn test_get_definitions() {
    let definitions = get_definitions(
        "https://raw.githubusercontent.com/elmarco/qemu/qapi/qapi-schema.json",
    ).unwrap();
    println!("use call_qemu;");
    for d in definitions {
        match d {
            Value::Object(map) => {
                // Most things should be objects
                if map.contains_key("struct") {
                    print_struct(map);
                } else if map.contains_key("command") {
                    print_command(map);
                } else if map.contains_key("enum") {
                    //print_enum(map);
                }
            }
            _ => {
                // Other value detected
                println!("Unknown value: {:?}", d);
            }
        }
    }
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

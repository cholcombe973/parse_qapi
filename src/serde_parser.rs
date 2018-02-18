extern crate heck;
extern crate nom;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use nom::IResult;
use self::heck::CamelCase;
use self::serde_json::Value;
use self::serde_json::Result as SerdeResult;

use {find_element, QemuType};

fn json_val_to_rust(input: &Value) -> String {
    match input {
        &Value::String(ref s) => match s.as_ref() {
            "uint64" => "u64".to_string(),
            "uint32" => "u32".to_string(),
            "bool" => "bool".to_string(),
            "int" => "f64".to_string(),
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

fn print_struct(v: serde_json::Map<String, Value>) -> Result<(), String> {
    let name = v.get("struct")
        .unwrap()
        .as_str()
        .unwrap();
    println!("#[derive(Debug)");
    println!("pub struct {} {{", name);
    for (field_name, field_type) in v.get("data")
        .unwrap()
        .as_object()
        .unwrap()
    {
        println!(
            "\tpub {}: {},",
            field_name.replace("*", "").replace("-", "_"),
            json_val_to_rust(field_type)
        );
    }
    println!("}}");
    Ok(())
}

fn print_command(v: HashMap<String, Value>) -> Result<(), String> {
    //
    Ok(())
}

fn print_enum(v: serde_json::Map<String, Value>) -> Result<(), String> {
    //enum: {"data": Array([String("read"), String("write")]), "enum": String("IoOperationType")}
    let name = v.get("enum")
        .unwrap()
        .as_str()
        .unwrap();
    println!("#[derive(Debug)]");
    println!("pub enum {} {{", name);
    // Enum can either contain an array of values or just a single value
    match v.get("data").unwrap(){
        &Value::Array(ref a) => {
            for field in a{
                println!("\t{},", field.as_str().unwrap().to_camel_case());
            }
        },
        _ => {
            return Err(format!("Unknown enum field: {:?}", v.get("data")));
        },
    };
    println!("}}");

    Ok(())
}

#[test]
fn test_get_definitions() {
    let definitions = get_definitions(
        "https://raw.githubusercontent.com/elmarco/qemu/qapi/qapi-schema.json",
    ).unwrap();
    for d in definitions {
        match d {
            Value::Object(map) => {
                // Most things should be objects
                if map.contains_key("struct") {
                    //println!("Struct: {:?}", map);
                    //print_struct(map);
                } else if map.contains_key("command") {
                    //println!("command: {:?}", map);
                } else if map.contains_key("enum") {
                    //println!("enum: {:?}", map);
                    print_enum(map);
                }
            }
            _ => {
                // Other value detected
                println!("Unknown value: {:?}", d);
            }
        }
    }
    //println!("definitions: {:#?}", definitions);
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

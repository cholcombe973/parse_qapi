#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;
extern crate serde;
extern crate serde_json;

use std::str::from_utf8;

mod serde_parser;
pub use serde_parser::generate_rust_definitions;

use nom::multispace;
use serde::de::DeserializeOwned;

use serde_json::Value;
use std::collections::HashMap;

// Qemu unfortunately has some variable names that are Rust reserved words
lazy_static! {
    static ref REPLACEMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("type".to_string(), "qemu_type".to_string());
        m.insert("in".to_string(), "qemu_in".to_string());
        m.insert("abstract".to_string(), "qemu_abstract".to_string());
        m.insert("String".to_string(), "qemu_string".to_string());
        m.insert("str".to_string(), "String".to_string());
        m.insert("int".to_string(), "i64".to_string());
        m.insert("0".to_string(), "qemu_0".to_string());
        m.insert("1".to_string(), "qemu_1".to_string());
        m.insert("2".to_string(), "qemu_2".to_string());
        m.insert("3".to_string(), "qemu_3".to_string());
        m.insert("4".to_string(), "qemu_4".to_string());
        m.insert("5".to_string(), "qemu_5".to_string());
        m.insert("6".to_string(), "qemu_6".to_string());
        m.insert("7".to_string(), "qemu_7".to_string());
        m.insert("8".to_string(), "qemu_8".to_string());
        m.insert("9".to_string(), "qemu_9".to_string());
        m
    };
}

fn call_qemu<T: DeserializeOwned>(v: serde_json::Value) -> Result<T, String> {
    let s = "";
    let result = serde_json::from_str::<T>(s).map_err(|e| e.to_string())?;

    Ok(result)
}

fn sanitize_name(name: &String) -> String {
    println!("sanitize_name: {}", name);
    let safe_name = name.replace("-", "_").replace("*", "").replace(".", "_");

    if REPLACEMAP.contains_key(&safe_name) {
        REPLACEMAP.get(&safe_name).unwrap().clone()
    } else {
        safe_name
    }
}

named!(blanks, chain!(many0!(multispace), || &b""[..]));

named!(comment_block<&[u8], Vec<String> >,
    chain!(
        comments: many0!(comment_line),
        ||{
            comments
        }
    )
);

named!(comment_line<&[u8], String>,
    chain!(
        tag!("#") ~
// alt!(
//    tag!("#")
//    | tag!("\n")
//    | tag!(" ")) ~
        line: map_res!(take_until_and_consume!("\n"), from_utf8),
        ||{
            line.to_string()
        }
    )
);

// Count {}'s and return a section
pub fn find_element(input: &[u8]) -> nom::IResult<&[u8], String> {
    let mut brace_count = 0;
    let mut count = 0;
    loop {
        if input.len() == 0 {
            break;
        }
        if input[count] as char == '{' {
            brace_count += 1;
            count += 1;
            continue;
        } else if input[count] as char == '}' {
            brace_count -= 1;
            count += 1;
            if brace_count == 0 {
                break;
            }
            continue;
        }
        count += 1;
    }
    nom::IResult::Done(
        &input[count..],
        String::from_utf8_lossy(&input[0..count]).into_owned(),
    )
}

fn remove_comments(input: String) -> String {
    let mut buf = String::new();
    let mut comment = false;
    for c in input.chars() {
        // I found the start of a comment line
        if c == '#' {
            comment = true;
        }
        if comment {
            // This is the end of the comment line
            if c == '\n' {
                comment = false;
            }
            // Continue until we reach the end of the line
            continue;
        }
        buf.push(c)
    }
    buf
}

#[test]
fn test_remove_comments() {
    let input = r#"{ 'union': 'ChardevBackend', 'data': { 'file'   : 'ChardevFile',
                                       'serial' : 'ChardevHostdev',
                                       'parallel': 'ChardevHostdev',
                                       'pipe'   : 'ChardevHostdev',
                                       'socket' : 'ChardevSocket',
                                       'udp'    : 'ChardevUdp',
                                       'pty'    : 'ChardevCommon',
                                       'null'   : 'ChardevCommon',
                                       'mux'    : 'ChardevMux',
                                       'msmouse': 'ChardevCommon',
                                       'braille': 'ChardevCommon',
                                       'testdev': 'ChardevCommon',
                                       'stdio'  : 'ChardevStdio',
                                       'console': 'ChardevCommon',
                                       'spicevmc' : 'ChardevSpiceChannel',
                                       'spiceport' : 'ChardevSpicePort',
                                       'vc'     : 'ChardevVC',
                                       'ringbuf': 'ChardevRingbuf',
                                       # next one is just for compatibility
                                       'memory' : 'ChardevRingbuf' } }"#;
    println!("{}", remove_comments(input.to_string()));
}

// named!(description<Description>,
// chain!(
// name: take_until_and_consume!("\n")~
// description: take_until!("@")~
//
// ||{
//
// }
// )
// )
//

#[test]
fn test_comment_parsing() {
    let x: &[u8] = &[];

    let input = r#"##
# @query-vnc:
#
# Returns information about the current VNC server
#
# Returns: @VncInfo
#
# Since: 0.14.0
##"#;
    let result = comment_block(input.as_bytes());
    println!("test_comment_parsing Result: {:?}", result);
}

#[derive(Debug, Eq, PartialEq)]
pub struct Description {
    pub name: String,
    pub parameters: Option<Vec<(String, String)>>,
    pub returns: Option<String>,
    pub version_since: String,
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Value,
    pub base: Value,
}

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
        &Value::Number(_) => "f64".to_string(),
        &Value::Bool(_) => "bool".to_string(),
        &Value::Null => "None".to_string(),
        &Value::Object(_) => "struct".to_string(),
        &Value::Array(_) => "Vec<String>".to_string(),
        //&Value::Number(s) => "".to_string(),
    }
}

impl Struct {
    fn parse(input: &Value) -> Self {
        // Check if base is first. Sometimes it comes first and sometimes data comes first
        Struct {
            name: input["struct"].as_str().unwrap().to_string(),
            fields: input["data"].clone(),
            base: input["base"].clone(),
        }
    }
    pub fn to_rust_string(self) -> String {
        let mut struct_fields: Vec<String> = Vec::new();

        if self.base.is_string() {
            struct_fields.push(format!("base: {}", self.base.as_str().unwrap()));
        }

        if self.fields.is_object() {
            for (key, value) in self.fields.as_object().unwrap() {
                let name = sanitize_name(key);
                struct_fields.push(format!("pub {name}:{type}",name=name,
                    type=json_val_to_rust(value)
                ));
            }
        }

        format!(
            r#"
            #[derive(Debug, RustcDecodable)]
            pub struct {name}{{
                {fields}
            }}
            "#,
            name = sanitize_name(&self.name),
            fields = struct_fields.join(",")
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: String,
    pub fields: Value,
    pub gen: Value,
    pub returns: Value,
}

impl Command {
    fn parse(input: &Value) -> Self {
        Command {
            name: input["command"].as_str().unwrap().to_string(),
            gen: input["gen"].clone(),
            fields: input["data"].clone(),
            returns: input["returns"].clone(),
        }
    }
    // TODO Put this in a mod of just qemu commands
    pub fn to_rust_string(self) -> String {
        let mut struct_fields: Vec<String> = Vec::new();
        let mut impl_fields: Vec<String> = Vec::new();
        let mut impl_input: Vec<String> = Vec::new();
        let mut returns = String::new();
        let mut to_json: Vec<String> = Vec::new();

        if self.fields.is_object() {
            for (key, value) in self.fields.as_object().unwrap() {
                let name = sanitize_name(key);
                let field_type = json_val_to_rust(value);

                to_json.push(format!(
                    "to_json[\"arguments\"][\"{qemu_name}\"] = self.{name}.clone().into();",
                    qemu_name = key.replace("*", ""),
                    name = name
                ));
                struct_fields.push(format!("pub {name}:{type}", name=name, type=field_type));
                impl_fields.push(format!("{name}:{name}", name = name));
                impl_input.push(format!("{name}:{type}",name=name, type=field_type));
            }
        }

        if !self.gen.is_null() {
            struct_fields.push("gen: bool".to_string());
            impl_input.push("gen: bool".to_string());
            impl_fields.push("gen: gen".to_string());
        }

        if !self.returns.is_null() {
            // This goes in the parse_qemu_response function
            match self.returns {
                Value::String(s) => {
                    let name = sanitize_name(&s);
                    returns.push_str(&format!(
                        r#"
                        fn parse_qemu_response(&self, response: &String) ->
                        rustc_json::DecodeResult<{name}>{{
                        rustc_json::decode(&response)
                        }}
                        "#,
                        name = name
                    ));
                }
                Value::Array(array) => {
                    let name = array.clone().pop();
                    match name {
                        Some(n) => {
                            returns.push_str(&format!(
                                r#"
                            fn parse_qemu_response(&self, response: &String) ->
                            rustc_json::DecodeResult<{name}>{{
                            rustc_json::decode(&response)
                            }}
                            "#,
                                name = n
                            ));
                        }
                        None => {
                            // TODO: What should we do here if the array doesn't have a value?
                        }
                    }
                }
                _ => {}
            };
        } else {
            let name = sanitize_name(&self.name);
            returns.push_str(&format!(
                r#"
                fn parse_qemu_response(&self, response: &String) ->
                rustc_json::DecodeResult<{name}>{{
                rustc_json::decode(&response)
                }}
                "#,
                name = name
            ));
        }

        format!(
            r#"
        #[derive(Debug)]
        pub struct {name} {{
            {fields}
        }}
        impl {name} {{
            pub fn new({impl_input})->{name}{{
                {name}{{
                    {impl_fields}
                }}
            }}
        }}
        impl QemuCmd for {name} {{
            fn to_json(&self)->String{{
                let mut to_json = json::Value::new_object();
                to_json["execute"] = "{execute_name}".into();
                {to_json_fields}
                to_json.dump()
            }}
        }}
        "#,
            name = sanitize_name(&self.name),
            execute_name = &self.name,
            fields = struct_fields.join(","),
            impl_fields = impl_fields.join(","),
            impl_input = impl_input.join(","),
            to_json_fields = to_json.join("\n") //,
                                                //parse_response=returns
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Union {
    pub name: String,
    pub discriminator: Value,
    pub data: Value,
}

impl Union {
    fn parse(input: &Value) -> Self {
        Union {
            name: input["union"].as_str().unwrap().to_string(),
            discriminator: input["discriminator"].clone(),
            data: input["data"].clone(),
        }
    }
    pub fn to_rust_string(self) -> String {
        let mut struct_fields: Vec<String> = Vec::new();

        if self.data.is_object() {
            for (key, value) in self.data.as_object().unwrap() {
                //for f in self.data.entries() {
                if key == "type" {
                    struct_fields.push(format!("pub qemu_type:{type}",
                        type=json_val_to_rust(value)
                    ));
                } else {
                    struct_fields.push(format!("pub {name}:{type}",
                        name=sanitize_name(key),
                        type=json_val_to_rust(value)
                    ));
                }
            }
        }

        format!(
            r#"
            #[derive(Debug,RustcDecodable)]
            pub struct {name}{{
                {fields}
            }}
            "#,
            name = self.name,
            fields = struct_fields.join(",")
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub name: String,
    pub data: Value,
}

impl Event {
    fn parse(input: &Value) -> Self {
        Event {
            name: input["event"].as_str().unwrap().to_string(),
            data: input["data"].clone(),
        }
    }
    pub fn to_rust_string(self) -> String {
        let mut struct_fields: Vec<String> = Vec::new();

        if self.data.is_object() {
            for (key, value) in self.data.as_object().unwrap() {
                let name = sanitize_name(key);
                let field_type = json_val_to_rust(value);

                struct_fields.push(format!("pub {name}:{type}", name=name, type=field_type));
            }
        }

        format!(
            r#"
            #[derive(Debug)]
            pub struct {name} {{
                {fields}
            }}
            "#,
            name = sanitize_name(&self.name),
            fields = struct_fields.join(","),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub fields: Value,
}

impl Enum {
    fn parse(input: &Value) -> Self {
        Enum {
            name: input["enum"].as_str().unwrap().to_string(),
            fields: input["data"].clone(),
        }
    }
    pub fn to_rust_string(self) -> String {
        let mut struct_fields: Vec<String> = Vec::new();

        if self.fields.is_array() {
            for f in self.fields.as_array().unwrap() {
                match f {
                    &Value::String(ref s) => {
                        let name = sanitize_name(s);
                        struct_fields.push(format!("{name}", name = name));
                    }
                    _ => {}
                }
            }
        }

        format!(
            r#"
            #[derive(Debug,RustcDecodable)]
            pub enum {name} {{
                {fields}
            }}
            "#,
            name = sanitize_name(&self.name),
            fields = struct_fields.join(","),
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum QemuType {
    Struct(Struct),
    Command(Command),
    Enum(Enum),
    Include { name: String },
    Event(Event),
    Union(Union),
    Unknown,
}

impl QemuType {
    fn parse(input: Value) -> Self {
        if !input["include"].is_null() {
            QemuType::Include {
                name: input["input"].as_str().unwrap_or("").to_string(),
            }
        } else if !input["enum"].is_null() {
            QemuType::Enum(Enum::parse(&input))
        } else if !input["command"].is_null() {
            QemuType::Command(Command::parse(&input))
        } else if !input["union"].is_null() {
            QemuType::Union(Union::parse(&input))
        } else if !input["struct"].is_null() {
            QemuType::Struct(Struct::parse(&input))
        } else if !input["event"].is_null() {
            QemuType::Event(Event::parse(&input))
        } else {
            QemuType::Unknown
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub description: Vec<String>,
    pub qemu_type: QemuType,
}

impl Section {
    fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        // println!("Section parse input: {:?}", String::from_utf8_lossy(input));
        chain!(
            input,
            comments: comment_block~
            element: call!(find_element) ~
            blanks?,
            ||{
                let result = if element.contains("#"){
                    let clean_element = remove_comments(element);
                    serde_json::from_str(&clean_element)
                }else{
                    serde_json::from_str(&element)
                };
                println!("Json result: {:?}", result);

                Section{
                    description: comments,
                    qemu_type: QemuType::parse(result.unwrap()),
                }
            }
        )
    }
}

pub fn parse_sections(input: &[u8]) -> nom::IResult<&[u8], Vec<Section>> {
    chain!(
        input,
        comment_block ~ //Get rid of the Mode: Python crap at the top
        sections: many0!(call!(Section::parse)),
        ||{
            sections
        }
    )
}

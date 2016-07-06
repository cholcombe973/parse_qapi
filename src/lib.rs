extern crate json;
#[macro_use] extern crate nom;

use std::str::{from_utf8};

use json::JsonValue;
use nom::{multispace};

named!(blanks,
       chain!(
           many0!(multispace),
           || { &b""[..] }));

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
        alt!(
            tag!("#")
            | tag!("\n")
            | tag!(" ")) ~
        line: map_res!(take_until_and_consume!("\n"), from_utf8),
        ||{
            line.to_string()
        }
    )
);

//Count {}'s and return a section
fn find_element(input: &[u8]) ->nom::IResult<&[u8], String>{
    let mut brace_count = 0;
    let mut count = 0;
    loop {
        if input[count] as char == '{'{
            brace_count +=1;
            count += 1;
            continue;
        }
        else if input[count] as char == '}'{
            brace_count -=1;
            count += 1;
            if brace_count == 0{
                break;
            }
            continue;
        }
        count += 1;
    }
    nom::IResult::Done(&input[count .. ], String::from_utf8_lossy(&input[0..count]).into_owned())
}

/*
named!(description<Description>,
    chain!(
        name: take_until_and_consume!("\n")~
        description: take_until!("@")~

        ||{

        }
    )
)
*/

#[test]
fn test_comment_parsing(){
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
pub struct Description{
    pub name: String,
    pub parameters: Option<Vec<(String,String)>>,
    pub returns: Option<String>,
    pub version_since: String,
}

#[derive(Debug, PartialEq)]
pub struct Struct{
    pub name: String,
    pub fields: JsonValue,
    pub base: JsonValue,
}

impl Struct{
    fn parse(input: &JsonValue) -> Self {
        //Check if base is first. Sometimes it comes first and sometimes data comes first
        Struct{
            name: input["struct"].as_str().unwrap_or("").to_string(),
            fields: input["data"].clone(),
            base: input["base"].clone(),
        }
    }
    pub fn to_string(self)->String{
        /*
        let mut struct_fields:Vec<String> = Vec::new();

        if let Some(b) = self.base{
            struct_fields.push(format!("base: {}", b));
        };

        for f in self.fields.clone(){
            struct_fields.push(format!("pub {name}:{type}",
                name=f.0.replace("-", "_"),
                type=f.1.replace("str", "String")
                .replace("int", "i64")
            ));
        }


        format!(r#"
            #[derive(Debug, Serialize, Deserialize)]
            pub struct {name}{{
                {fields}
            }}
            "#, name=self.name, fields=struct_fields.join(","))
        */
        "".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct Command{
    pub name: String,
    pub fields: JsonValue,
    pub gen: JsonValue,
    pub returns: JsonValue,
}

impl Command{
    fn parse(input: &json::JsonValue) -> Self {
        Command{
            name: input["command"].as_str().unwrap_or("").to_string(),
            gen: input["gen"].clone(),
            fields: input["data"].clone(),
            returns: input["returns"].clone(),
        }
    }
    //TODO Put this in a mod of just qemu commands
    pub fn to_string(self)->String{
        /*
        let mut struct_fields:Vec<String> = Vec::new();
        let mut impl_fields:Vec<String> = Vec::new();
        let mut impl_input:Vec<String> = Vec::new();

        if let Some(f) = self.fields{
            for field in f{
                let name = field.0.replace("-", "_");
                let field_type =field.1.replace("str", "String")
                    .replace("int", "i64");

                struct_fields.push(format!("pub {name}:{type}", name=name, type=field_type));
                impl_fields.push(format!("{name}:{name}", name=name));
                impl_input.push(format!("{name}:{type}",name=name, type=field_type));
            }
        }

        if let Some(r) = self.returns{
            match r{
                QemuReturnType::List(l) => {
                    struct_fields.push(format!("#[serde(skip_serializing)]\nreturns:{}", l.replace("str", "String")));
                    impl_fields.push(format!("returns:{}", l.replace("str", "String")));
                    impl_input.push(format!("{name}:Vec<{type}>",name=l, type=l));
                },
                QemuReturnType::String(s) => {
                    struct_fields.push(format!("#[serde(skip_serializing)]\nreturns:{}", s.replace("str", "String")));
                    impl_fields.push(format!("returns:{}", s.replace("str", "String")));
                    impl_input.push(format!("{name}:{type}",name=s, type=s));
                },
            }
        }

        let mut gen = String::new();
        if let Some(g) = self.gen{
            struct_fields.push("gen: bool".to_string());
        }

        format!(r#"
        #[derive(Debug, Serialize, Deserialize)]
        pub struct {name} {{
            execute: String,
            {fields}
        }}
        impl {name} {{
            pub fn new({impl_input})->{name}{{
                {name}{{
                    execute: "{name}".to_string(),
                    {impl_fields}
                }}
            }}
        }}
        "#,
        name=self.name.replace("-", "_"),
        fields=struct_fields.join(","),
        impl_fields=impl_fields.join(","),
        impl_input=impl_input.join(",")
    )
    */
    "".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct Union{
    pub name: String,
    pub discriminator: JsonValue,
    pub data: JsonValue,

}

impl Union{
    fn parse(input: &json::JsonValue) -> Self {
        Union{
            name: input["union"].as_str().unwrap_or("").to_string(),
            discriminator: input["discriminator"].clone(),
            data: input["data"].clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Event{
    pub name: String,
    pub data: JsonValue,
}

impl Event{
    fn parse(input: &json::JsonValue) -> Self {
        Event{
            name: input["event"].as_str().unwrap_or("").to_string(),
            data: input["data"].clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Enum{
    pub name: String,
    pub fields: json::JsonValue,
}

impl Enum{
    fn parse(input: &json::JsonValue) -> Self {
        Enum{
            name: input["enum"].as_str().unwrap_or("").to_string(),
            fields: input["data"].clone(),
        }
    }
    pub fn to_string(self)->String{
        /*
        format!(r#"
            #[derive(Debug, Serialize, Deserialize)]
            pub enum {} {{
                {fields}
            }}"#, self.name, fields=self.fields.into_iter().map(|s| s.replace("-", "_")).collect::<Vec<String>>().join(","))
        */
        "".to_string()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum QemuReturnType{
    List(String), //A list type
    String(String), // Returns a single thing
}

#[derive(Debug, PartialEq)]
pub enum QemuType{
    Struct(Struct),
    Command(Command),
    Enum(Enum),
    Include{
        name: String,
    },
    Event(Event),
    Union(Union),
    Unknown,
}

impl QemuType {
    fn parse(input: json::JsonValue) -> Self {
        if ! input["include"].is_null(){
            QemuType::Include{
                name: input["input"].as_str().unwrap_or("").to_string()
            }
        }else if ! input["enum"].is_null(){
            QemuType::Enum(Enum::parse(&input))
        }else if ! input["command"].is_null(){
            QemuType::Command(Command::parse(&input))

        }else if ! input["union"].is_null(){
            QemuType::Union(Union::parse(&input))

        }else if ! input["struct"].is_null(){
            QemuType::Struct(Struct::parse(&input))
        }else if ! input["event"].is_null(){
            QemuType::Event(Event::parse(&input))
        }else{
            QemuType::Unknown
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub description: Vec<String>,
    pub qemu_type: QemuType,
}

impl Section{
    fn parse(input: &[u8]) -> nom::IResult<&[u8], Self>{
        //println!("Section parse input: {:?}", String::from_utf8_lossy(input));
        chain!(
            input,
            comments: comment_block~
            element: call!(find_element) ~
            blanks?,
            ||{
                let result = json::parse(&element).unwrap();
                println!("Json result: {:?}", result);

                Section{
                    description: comments,
                    qemu_type: QemuType::parse(result),
                }
            }
        )
    }
}

pub fn print_section(s: Section){
    match s.qemu_type{
        QemuType::Struct(st) => {
            //TODO: Write these to structs/mod.rs
            println!("{}", s.description.join("\n///"));
            println!("{}", st.to_string());
        },
        QemuType::Command(c) => {
            println!("{}", s.description.join("\n///"));
            println!("{}", c.to_string());
        },
        QemuType::Enum(e) => {
            println!("{}", s.description.join("\n///"));
            println!("{}", e.to_string());
        },
        QemuType::Include{name} => {
            println!("{}", s.description.join("\n///"));
            println!("//{}", name.to_string());
        },
        QemuType::Event(event) => {

        },
        QemuType::Union(u) => {

        },
        QemuType::Unknown => {},
    }
}

pub fn parse_sections(input: &[u8])-> nom::IResult<&[u8], Vec<Section>>{
    chain!(
        input,
        comment_block ~ //Get rid of the Mode: Python crap at the top
        sections: many0!(call!(Section::parse)),
        ||{
            sections
        }
    )
}

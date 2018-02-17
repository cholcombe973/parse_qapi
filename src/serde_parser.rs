extern crate serde;
extern crate serde_json;

use self::serde_json::Value;

use find_element;

#[test]
fn test_parser() {
    extern crate nom;
    extern crate reqwest;

    use self::serde_json::Result as SerdeResult;

    use nom::IResult;

    let text = reqwest::get("https://raw.githubusercontent.com/elmarco/qemu/qapi/qapi-schema.json").unwrap()
    .text().unwrap();
    let mut buff = String::new();
    for line in text.lines(){
        if !line.starts_with("#"){
            // For lines that have a comment in the middle of the line
            // we remove the # until \n
            if line.contains("#"){
                let v: Vec <&str> = line.split("#").collect();
                buff.push_str(v[0].into());
            }else{
                buff.push_str(line);
            }
        }
    }
    let mut elements: Vec<String> = Vec::new();
    let mut leftover = buff.as_bytes();
    loop {
        match find_element(leftover){
            IResult::Done(left, s) => {
                if s.is_empty(){
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
    //println!("elements: {:?}", elements);
    for e in elements {
        //println!("parse: {}", e);
        let s = e.replace("'", "\"");
        let v: SerdeResult<Value> = serde_json::from_str(&s);
        println!("v: {:?}", v);
    }
}

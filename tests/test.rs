extern crate nom;
extern crate parse_qapi;

use std::fs::File;
use std::io::prelude::*;

#[test]
fn test_block() {
    let mut f = File::open("tests/block.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    // Sanitize input
    let buf = buf.replace("\'", "\"");

    let result = parse_qapi::parse_sections(buf.as_bytes());
    // println!("Block Result: {:?}", result);
    match result {
        nom::IResult::Done(left, qemu_sections) => {
            println!("Block leftover: {}", String::from_utf8_lossy(left));
            println!("Block Result: {:?}", qemu_sections);
        }
        nom::IResult::Incomplete(needed) => {
            println!("Incomplete: {:?}", needed);
        }
        nom::IResult::Error(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[test]
fn test_block_core() {
    let mut f = File::open("tests/block-core.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    // Sanitize input
    let buf = buf.replace("\'", "\"");

    let result = parse_qapi::parse_sections(buf.as_bytes());
    // println!("Block Result: {:?}", result);
    match result {
        nom::IResult::Done(left, qemu_sections) => {
            println!("Block leftover: {}", String::from_utf8_lossy(left));
            println!("Block Result: {:?}", qemu_sections);
        }
        nom::IResult::Incomplete(needed) => {
            println!("Incomplete: {:?}", needed);
        }
        nom::IResult::Error(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[test]
fn test_common() {
    let mut f = File::open("tests/common.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    // Sanitize input
    let buf = buf.replace("\'", "\"");

    let result = parse_qapi::parse_sections(buf.as_bytes());
    match result {
        nom::IResult::Done(left, qemu_sections) => {
            println!("Common leftover: {}", String::from_utf8_lossy(left));
            println!("Common Result: {:?}", qemu_sections);
        }
        nom::IResult::Incomplete(needed) => {
            println!("Incomplete: {:?}", needed);
        }
        nom::IResult::Error(e) => {
            println!("Error: {:?}", e);
        }
    }
    // println!("Common Result: {:?}", result);
}

#[test]
fn test_qapi() {
    let mut f = File::open("tests/qapi.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    // Sanitize input
    let buf = buf.replace("\'", "\"");

    let result = parse_qapi::parse_sections(buf.as_bytes());
    match result {
        nom::IResult::Done(left, qemu_sections) => {
            println!("QAPI leftover: {}", String::from_utf8_lossy(left));
            println!("QAPI Result: {:?}", qemu_sections);
        }
        nom::IResult::Incomplete(needed) => {
            println!("Incomplete: {:?}", needed);
        }
        nom::IResult::Error(e) => {
            println!("Error: {:?}", e);
        }
    }
    // println!("QAPI Result: {:?}", result);
}

#[test]
fn test_event() {
    let mut f = File::open("tests/event.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    // Sanitize input
    let buf = buf.replace("\'", "\"");

    let result = parse_qapi::parse_sections(buf.as_bytes());
    match result {
        nom::IResult::Done(left, qemu_sections) => {
            println!("Event leftover: {}", String::from_utf8_lossy(left));
            println!("Event Result: {:?}", qemu_sections);
        }
        nom::IResult::Incomplete(needed) => {
            println!("Incomplete: {:?}", needed);
        }
        nom::IResult::Error(e) => {
            println!("Error: {:?}", e);
        }
    }
    // println!("QAPI Result: {:?}", result);
}

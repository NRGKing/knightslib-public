use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use json::JsonValue;

use crate::util::Movement;

pub fn write_to_file(path: String, content: String) -> Result<(), String> {
    let path = Path::new(&path);
    let mut file = match File::create(&path) {
        Err(error) => return Err(format!("Couldn't create file: {}", error)),
        Ok(file) => file,
    };

    match file.write_all(content.as_bytes()) {
        Err(error) => return Err(format!("Couldn't write to file: {}", error)),
        Ok(_) => Ok(()),
    }
}

pub fn dump_route(path: String, route: Vec<Movement>) {
    // write_to_file(path, route.to_json());

    let json_array: Vec<JsonValue> = route.iter().map(|item| item.to_json()).collect();

    // wrap it in a JSON object if needed
    let wrapped_json = object! {
        "items" => json_array,
    };

    println!("{}", wrapped_json);
    let _ = write_to_file(path, wrapped_json.to_string());
}

pub fn read_route(path: String) -> Result<Vec<Movement>, String> {
    println!("Reached Read Route");

    let file_json: String = match std::fs::read_to_string(path) {
        Ok(string) => string,
        Err(error) => return Err(format!("Couldnt read file: {}", error)),
    };

    println!("File String: {:?}", file_json);

    // let json_read: Result<Vec<Movement>, serde_json::Error>= serde_json::from_str(&file_json);
    let json_read = json::parse(&file_json).unwrap();

    println!("Parsed Json String: {:?}", json_read);

    let movements = Movement::from_json_array(&json_read["items"]);

    println!("Json Result: {:?}", movements);

    return Ok(movements.unwrap());
}

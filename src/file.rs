use std::fs::File;

use serde::{de, Serialize};

pub fn write_to_file<T: Serialize>(filename: &str, data: T) {
    let file = match File::open(&filename) {
        Ok(file) => file,
        Err(_) => File::create(&filename).unwrap(),
    };
    serde_cbor::to_writer(file, &data).unwrap();
}

pub fn exists_file(filename: &str) -> bool {
    match File::open(&filename) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn read_from_file<T: de::DeserializeOwned>(filename: &str) -> T {
    let file = File::open(&filename).unwrap();
    match serde_cbor::from_reader(file) {
        Ok(data) => data,
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

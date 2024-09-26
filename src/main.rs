// use regex;
use std::{
    env::args, error::Error, fs::{read, read_dir, read_to_string}
};

#[derive(Clone, Copy)]
enum StrEncoding {
    Utf8,
    Utf16
}

struct StrUtf816 {
    str: String,
    encoding_type: StrEncoding
}

impl StrUtf816 {
    pub fn as_str<'a>(&'a self) -> &'a str {
        self.str.as_str()
    }
    
    pub fn encoding(&self) -> StrEncoding {
        self.encoding_type
    }
    
    pub fn set_encoding(&mut self, value: StrEncoding) {
        self.encoding_type = value; 
    }
}

impl From<StrUtf816> for String {
    fn from(value: StrUtf816) -> String {
        value.str
    }
}

impl AsRef<str> for StrUtf816 {
    fn as_ref(&self) -> &str {
        &self.as_str()
    }
}

impl From<StrUtf816> for Vec<u8> {
    fn from(value: StrUtf816) -> Vec<u8> {
        match value.encoding() {
            StrEncoding::Utf8 => {
                String::from(value).bytes().collect()
            }, 
            StrEncoding::Utf16 => {
                String::from(value).encode_utf16().flat_map(|x| x.to_ne_bytes()).collect()
            }
        }
    }
}

fn main() {
    let path = args()
        .nth(1)
        .expect("Expected 1 argument, none were given");

    let result = print_dirs(path);

    if let Err(e) = result {
        println!("{}", e);
    }
}

fn print_dirs(path: String) -> Result<(), Box<dyn Error>> {
    let dir = read_dir(path);
    match dir {
        Ok(d) => {
            for i in d {
                match i {
                    Ok(f) => {
                        println!(
                            "Currently at '{}'",
                            f.path().as_os_str().to_string_lossy().to_string()
                        );
                        if f.path().is_file() {
                            println!(
                                "{}",
                                read_utf8_utf16(
                                    f.path().as_os_str().to_str().unwrap()
                                )?
                            );
                        } else if f.path().is_dir() {
                            print_dirs(f.path().to_string_lossy().into())?;
                        }
                    }
                    Err(e) => return Err(Box::new(e)),
                }
            }
        }
        Err(e) => return Err(Box::new(e)),
    }

    Ok(())
}

fn read_utf8_utf16(path: &str) -> Result<String, std::io::Error> {
    let content = read_to_string(path);
    match content {
        Ok(c) => Ok(c),
        Err(_) => {
            let bytes = read(path)?;
            Ok(String::from_utf16_lossy(
                bytes
                    .chunks_exact(2)
                    .map(|b| u16::from_ne_bytes([b[0], b[1]]))
                    .collect::<Vec<u16>>()
                    .as_slice(),
            ))
        }
    }
}

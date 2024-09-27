// use regex;
use std::{
    env::args,
    error::Error,
    fs::{read, read_dir, read_to_string, write},
};

#[derive(Clone, Copy)]
enum StrEncoding {
    Utf8,
    Utf16,
}

struct StrUtf816 {
    str: String,
    encoding_type: StrEncoding,
}

impl StrUtf816 {
    pub fn as_str<'a>(&'a self) -> &'a str {
        &self.str
    }

    #[inline]
    pub fn new(text: String, encoding: StrEncoding) -> StrUtf816 {
        StrUtf816 {
            str: text,
            encoding_type: encoding,
        }
    }

    pub fn encoding(&self) -> StrEncoding {
        self.encoding_type
    }

    #[allow(dead_code)]
    pub fn set_encoding(&mut self, value: StrEncoding) {
        self.encoding_type = value;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn default() -> StrUtf816 {
        StrUtf816 {
            str: "".into(),
            encoding_type: StrEncoding::Utf8,
        }
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
            StrEncoding::Utf8 => String::from(value).bytes().collect(),
            StrEncoding::Utf16 => String::from(value)
                .encode_utf16()
                .flat_map(|x| x.to_ne_bytes())
                .collect(),
        }
    }
}

impl std::ops::Deref for StrUtf816 {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.str
    }
}

impl std::fmt::Display for StrUtf816 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.str.fmt(f)
    }
}

impl AsMut<String> for StrUtf816 {
    fn as_mut(&mut self) -> &mut String {
        &mut self.str
    }
}

fn main() {
    let mut args = args().skip(1);
    let mut replacemnts_path: Option<String> = None;
    let mut open_path: Option<String> = None;
    let mut save_path: Option<String> = None;
    while let Some(a) = &args.next() {
        match a.as_str() {
            "--replacements" => replacemnts_path = args.next(),
            "--open" => open_path = args.next(),
            "--save" => save_path = args.next(),
            _ => unreachable!(),
        }
    }

    let result = find_replace_dir(
        get_replacements(&replacemnts_path.unwrap()).unwrap(),
        &open_path.unwrap(),
        &save_path.unwrap(),
    );

    if let Err(e) = result {
        println!("{}", e);
    }
}

#[allow(dead_code)]
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
                                read_utf8_utf16(f.path().as_os_str().to_str().unwrap())?
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

/// Reads the specified file to a string, accepting utf-8 and utf-16 encoded strings as input
fn read_utf8_utf16(path: &str) -> Result<StrUtf816, std::io::Error> {
    let content = read_to_string(path);
    match content {
        Ok(c) => Ok(StrUtf816::new(c, StrEncoding::Utf8)),
        Err(_) => {
            let bytes = read(path)?;
            Ok(StrUtf816::new(
                String::from_utf16_lossy(
                    bytes
                        .chunks_exact(2)
                        .map(|b| u16::from_ne_bytes([b[0], b[1]]))
                        .collect::<Vec<u16>>()
                        .as_slice(),
                ),
                StrEncoding::Utf16,
            ))
        }
    }
}

/// Writes the input text to the specified file encoded as either utf-8 or utf-16 as required
fn write_utf8_utf16(path: &str, text: StrUtf816) -> Result<(), std::io::Error> {
    match text.encoding() {
        StrEncoding::Utf8 => write(path, text.as_str())?,
        StrEncoding::Utf16 => write(
            path,
            text.encode_utf16()
                .flat_map(|x| x.to_ne_bytes())
                .collect::<Vec<u8>>()
                .as_slice(),
        )?,
    }

    Ok(())
}

/// bulk in-place find and replace in text
fn replace_all(find_replace: Vec<(String, String)>, text: &mut String) {
    for value in find_replace {
        loop {
            if let Some(start) = text.find(&value.0) {
                let end = start + value.0.len();
                text.replace_range(start..end, &value.1);
                continue;
            }

            break;
        }
    }
}

/// Recursively searches through directories in at the specified path and does a find and replace to all files within.
/// Saves the modified files in a specified new location. Works with utf-8 and utf-16 encoded files
fn find_replace_dir(
    find_replace: Vec<(String, String)>,
    open_path: &str,
    write_path: &str,
) -> Result<(), std::io::Error> {
    let dir = read_dir(open_path);
    match dir {
        Ok(d) => {
            for entry in d {
                match entry {
                    Ok(f) => {
                        if f.file_type()?.is_file() {
                            let mut content = read_utf8_utf16(f.path().to_str().unwrap())?;
                            replace_all(find_replace.clone(), &mut content.as_mut());
                            write_utf8_utf16(&write_path, content)?;
                        } else if f.file_type()?.is_dir() {
                            let new_open_path =
                                format!("{}/{}", open_path, f.file_name().to_string_lossy());
                            let new_write_path =
                                format!("{}/{}", write_path, f.file_name().to_string_lossy());
                            std::fs::create_dir_all(&new_write_path)?;
                            find_replace_dir(
                                find_replace.clone(),
                                &new_open_path,
                                &new_write_path,
                            )?;
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not read directory",
            ))
        }
    }
    Ok(())
}

fn get_replacements(path: &str) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = read_to_string(path)?;
    Ok(content
        .lines()
        .filter_map(|l| l.split_once(','))
        .map(|t| (t.0.to_string(), t.1.to_string()))
        .collect())
}

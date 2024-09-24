fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Expected 1 argument, none were given");

    let result = print_dirs(path);

    if let Err(e) = result {
        println!("{}", e);
    }
}

fn print_dirs(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::fs::read_dir(path);
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
                                    f.path().as_os_str().to_str().unwrap().to_string()
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

fn read_utf8_utf16(path: String) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path.as_str());
    match content {
        Ok(c) => Ok(c),
        Err(_) => {
            let bytes = std::fs::read(path.as_str())?;
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

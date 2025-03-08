use shared::pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, Result},
    net::{TcpListener, TcpStream},
    str,
    sync::{Arc, Mutex},
};

struct Blob {
    pub revisions: Vec<String>,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(100);

    let storage = Arc::new(Mutex::new(HashMap::<String, Blob>::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let storage = Arc::clone(&storage);

        pool.execute(|_| {
            if let Err(e) = handle_connection(stream, storage) {
                eprintln!("Connection error: {}", e);
            }
        })
    }

    Ok(())
}

fn handle_connection(stream: TcpStream, storage: Arc<Mutex<HashMap<String, Blob>>>) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    writeln!(writer, "READY")?;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        match line.trim().to_lowercase().split(' ').collect::<Vec<&str>>()[..] {
            ["help"] => {
                writeln!(writer, "OK usage: HELP|GET|PUT|LIST")?;
            }
            ["put", file, length] => 'put: {
                let allowed_chars = |c: char| c.is_alphanumeric() || "/._-".contains(c);

                if !file.starts_with('/') || !file.chars().all(allowed_chars) {
                    writeln!(writer, "ERR illegal file name")?;
                    break 'put;
                }

                let size: usize = length.parse().unwrap();
                let mut buffer = vec![0; size];
                reader.read_exact(&mut buffer)?;

                if buffer
                    .iter()
                    .any(|&b| !(b.is_ascii_graphic() || b.is_ascii_whitespace()))
                {
                    writeln!(writer, "ERR invalid data")?;
                    break 'put;
                }

                let data = str::from_utf8(&buffer).unwrap();

                let mut storage = storage.lock().unwrap();

                if let Some(blob) = storage.get_mut(file) {
                    if data != blob.revisions[blob.revisions.len() - 1] {
                        blob.revisions.push(data.to_string());
                    }

                    writeln!(writer, "OK r{}", blob.revisions.len())?;
                } else {
                    let blob = Blob {
                        revisions: Vec::from([data.to_string()]),
                    };
                    storage.insert(file.to_string(), blob);

                    writeln!(writer, "OK r1")?;
                }
            }
            ["put", ..] => {
                writeln!(writer, "ERR usage: PUT file length newline data")?;
            }
            ["get", file] => 'get: {
                if !file.starts_with('/') {
                    writeln!(writer, "ERR illegal file name")?;
                    break 'get;
                }

                if let Some(blob) = storage.lock().unwrap().get_mut(file) {
                    writeln!(
                        writer,
                        "OK {}",
                        blob.revisions[blob.revisions.len() - 1].len()
                    )?;
                    write!(writer, "{}", blob.revisions[blob.revisions.len() - 1])?;
                } else {
                    writeln!(writer, "ERR no such file")?;
                }
            }
            ["get", file, revision] => 'get: {
                if !file.starts_with('/') {
                    writeln!(writer, "ERR illegal file name")?;
                    break 'get;
                }

                if let Some(blob) = storage.lock().unwrap().get_mut(file) {
                    let revision: usize = revision.replace("r", "").parse().unwrap_or(0);

                    if revision < 1 || revision > blob.revisions.len() {
                        writeln!(writer, "ERR no such revision")?;
                    } else {
                        writeln!(writer, "OK {}", blob.revisions[revision - 1].len())?;
                        write!(writer, "{}", blob.revisions[revision - 1])?;
                    }
                } else {
                    writeln!(writer, "ERR no such file")?;
                }
            }
            ["get", ..] => {
                writeln!(writer, "ERR usage: GET file [revision]")?;
            }
            ["list", dir] => 'list: {
                if !dir.starts_with('/') {
                    writeln!(writer, "ERR illegal dir name")?;
                    break 'list;
                }
                let directory = dir.trim_end_matches("/");

                let mut blobs: Vec<String> = Vec::new();

                for (key, blob) in storage.lock().unwrap().iter() {
                    match key.rsplitn(3, "/").collect::<Vec<&str>>()[..] {
                        [filename] if directory.is_empty() => {
                            blobs.push(format!("{} r{}", filename, blob.revisions.len()));
                        }
                        [filename, base_dir] => {
                            if base_dir == directory {
                                blobs.push(format!("{} r{}", filename, blob.revisions.len()));
                            }
                        }
                        [filename, sub_dir, base_dir] => {
                            if base_dir == directory {
                                blobs.push(format!("{}/ DIR", sub_dir));
                            }
                            if [base_dir, sub_dir].join("/") == directory {
                                blobs.push(format!("{} r{}", filename, blob.revisions.len()));
                            }
                        }
                        _ => {}
                    }
                }

                writeln!(writer, "OK {}", blobs.len())?;

                blobs.sort();
                for blob in blobs {
                    writeln!(writer, "{}", blob)?;
                }
            }
            ["list", ..] => {
                writeln!(writer, "ERR usage: LIST dir")?;
            }
            [first, ..] => {
                writeln!(writer, "ERR illegal method: {}", first)?;
                break;
            }
            [] => {
                writeln!(writer, "ERR illegal method: ")?;
                break;
            }
        }

        writeln!(writer, "READY")?;
    }

    Ok(())
}

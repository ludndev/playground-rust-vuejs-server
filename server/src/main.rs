use std::{env, fs};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;
use std::thread;
use std::collections::HashMap;  // Replace the first line with this combined import

const WEB_ROOT: &str = "./../web/dist";

fn handle_request(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    let binding = get_root_dir();
    let web_root = binding.as_str();
    
    let bytes_read = stream.read(&mut buffer)?;
    let request = str::from_utf8(&buffer[..bytes_read]).unwrap_or("");
    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 || parts[0] != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\nMethod not allowed!";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let requested_path_initial = parts[1].to_string();    
    println!("    [GET] 200 {}", requested_path_initial);

    // Parse query parameters
    let query_params: HashMap<String, String> = if let Some(query) = parts[1].split('?').nth(1) {
        query.split('&')
            .filter_map(|param| {
                let mut parts = param.split('=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                    _ => None,
                }
            })
            .collect()
    } else {
        HashMap::new()
    };

    println!("  > debug: query_params: {:?}", query_params);  // Fixed debug print format

    let mut requested_path = requested_path_initial.clone();
    
    // handle Next.js image route
    if requested_path.starts_with("/_next/image") {
        if let Some(url_param) = query_params.get("url") {
            requested_path = url_param.to_string();

            let replacements = [
                ("%25", "%"),  // percent sign
                ("%20", " "),  // space
                ("%2F", "/"),  // forward slash
                ("%3A", ":"),  // colon
                ("%2D", "-"),  // hyphen
                ("%2E", "."),  // period
                ("%5F", "_"),  // underscore
                ("%7E", "~"),  // tilde
                ("%2B", "+"),  // plus
                ("%23", "#"),  // hash
                ("%3F", "?"),  // question mark
                ("%26", "&"),  // ampersand
                ("%3D", "="),  // equals
                ("%40", "@"),  // at sign
                ("%24", "$"),  // dollar sign
            ];

            for (encoded, decoded) in replacements.iter() {
                requested_path = requested_path.replace(encoded, decoded);
            }

            // debug output with special handling for URLs
            if requested_path.starts_with("http://") || requested_path.starts_with("https://") {
                let response = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {}\r\nConnection: close\r\n\r\n",
                    requested_path
                );
                stream.write_all(response.as_bytes())?;
                return Ok(());
            } else {
                let default_width = "0".to_string();
                let default_quality = "100".to_string();
                let width = query_params.get("w").unwrap_or(&default_width);
                let quality = query_params.get("q").unwrap_or(&default_quality);
                println!("  > debug: image request - path: {}, width: {}, quality: {}", requested_path, width, quality);
            }
        }
    } else {
        requested_path = requested_path
            .split('?')
            .next()
            .unwrap_or("")
            .to_string();
    }
    
    if requested_path == "/" {
        requested_path = "/index.html".to_string();
    }

    let file_path = Path::new(web_root).join(requested_path.trim_start_matches('/'));

    let content_type = get_content_type(&file_path);

    let result = if file_path.exists() && file_path.is_file() {
        match fs::read(&file_path) {
            Ok(contents) => {
                let response_headers = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    content_type,
                    contents.len()
                );
                stream.write_all(response_headers.as_bytes())?;
                stream.write_all(&contents)
            },
            Err(e) => {
                eprintln!("500 Internal Server Error: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nError reading file";
                stream.write_all(response.as_bytes())
            }
        }
    } else {
        println!("    [GET] 200 {} (SPA Route)", requested_path);
        let index_path = Path::new(web_root).join("index.html");
        match fs::read(index_path) {
            Ok(contents) => {
                let response_headers = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    contents.len()
                );
                stream.write_all(response_headers.as_bytes())?;
                stream.write_all(&contents)
            },
            Err(e) => {
                eprintln!("500 Internal Server Error: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nError reading index.html";
                stream.write_all(response.as_bytes())
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error sending response: {}", e);
    }

    stream.flush()?;
    Ok(())
}

fn start_server(address: &str) -> io::Result<()> {
    let listener = TcpListener::bind(address)?;
    println!("Server running at http://{}\n", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::spawn(move || {
                    if let Err(e) = handle_request(stream) {
                        eprintln!("Error handling request: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn get_root_dir() -> String {
    // check if arg --dir is provided
    let args: Vec<String> = env::args().collect();
    if let Some(dir_arg_pos) = args.iter().position(|arg| arg == "--dir") {
        if dir_arg_pos + 1 < args.len() {
            return args[dir_arg_pos + 1].clone();
        }
    }
    
    // return default directory
    WEB_ROOT.to_string()
}

fn get_content_type(file_path: &Path) -> &'static str {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("mjs") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("md") => "text/markdown",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

fn main() {
    if let Err(e) = start_server("127.0.0.1:8080") {
        eprintln!("Error starting server: {}", e);
    }
}

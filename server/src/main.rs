use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;

const WEB_ROOT: &str = "./../web/dist";

fn handle_request(mut stream: TcpStream) -> io::Result<()> {
    #[allow(unused_mut)]
    let mut web_root = WEB_ROOT;

    let mut buffer = [0; 1024];

    println!("New connection received");
    
    let bytes_read = stream.read(&mut buffer)?;
    let request = str::from_utf8(&buffer[..bytes_read]).unwrap_or("");
    println!("Received request: {}", request);

    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 || parts[0] != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\nMethod not allowed!";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let mut requested_path = parts[1].to_string();
    // Remove query parameters if they exist
    requested_path = requested_path.split('?').next().unwrap_or("").to_string();
    
    if requested_path == "/" {
        requested_path = "/index.html".to_string();
    }

    let file_path = Path::new(web_root).join(requested_path.trim_start_matches('/'));
    println!("Attempting to read file: {:?}", file_path.to_string_lossy().replace("\\", "/"));

    // Get the file extension to determine content type
    let content_type = match file_path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };

    let result = if file_path.exists() && file_path.is_file() {
        match fs::read(&file_path) {
            Ok(contents) => {
                println!("File found, sending response");
                let response_headers = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    content_type,
                    contents.len()
                );
                
                stream.write_all(response_headers.as_bytes())?;
                stream.write_all(&contents)
            },
            Err(e) => {
                println!("Error reading file: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nError reading file";
                stream.write_all(response.as_bytes())
            }
        }
    } else {
        println!("File not found: {:?}", file_path.to_string_lossy().replace("\\", "/"));
        let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nFile not found!";
        stream.write_all(response.as_bytes())
    };

    if let Err(e) = result {
        eprintln!("Error sending response: {}", e);
    }

    if let Err(e) = stream.flush() {
        eprintln!("Error flushing stream: {}", e);
    }

    Ok(())
}

fn start_server(address: &str) -> io::Result<()> {
    let listener = TcpListener::bind(address)?;

    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_request(stream) {
                    eprintln!("Error handling request: {}", e);
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn main() {
    // start the server on address 127.0.0.1:8080
    if let Err(e) = start_server("127.0.0.1:8080") {
        eprintln!("Error starting server: {}", e);
    }
}

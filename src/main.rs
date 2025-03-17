use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename, content_type) = if request_line.starts_with("GET / ") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/html/home.html", "text/html")
    } else if request_line.starts_with("GET /header.html") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/partials/header.html", "text/html")
    } else if request_line.starts_with("GET /css/header.css") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/css/header.css", "text/css")
    } else if request_line.starts_with("GET /css/home.css") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/css/home.css", "text/css")
    } else if request_line.starts_with("GET /images/snakeheader.png") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/images/snakeheader.png", "image/png")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "../snakemicrobit/src/views/html/404.html", "text/html")
    };

    let path = Path::new(filename);

    if path.exists() && path.is_file() {
        if content_type == "image/png" {
            // Serve image as binary data
            let contents = fs::read(filename).unwrap_or_else(|_| vec![]);
            let response = format!(
                "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                status_line,
                contents.len(),
                content_type
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&contents).unwrap(); // Send image binary data
        } else {
            // Serve text-based files (HTML, CSS, etc.)
            let contents = fs::read_to_string(filename).unwrap_or_else(|_| "<h1>404 - File Not Found</h1>".to_string());
            let response = format!(
                "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
                status_line,
                contents.len(),
                content_type,
                contents
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    } else {
        // File not found
        let not_found = "<h1>404 - File Not Found</h1>".to_string();
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
            not_found.len(),
            not_found
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}

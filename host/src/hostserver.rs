

use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, path::Path, process::Command, thread};

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
    let maybe_request_line = buf_reader.lines().next();

    let request_line = match maybe_request_line {
        Some(Ok(line)) => line,
        _ => {
            eprintln!("⚠️  Client disconnected or sent invalid request.");
            return;
        }
    };

    println!("📥 Incoming request: {}", request_line);


 /*   if request_line.starts_with("GET /flash") {
        thread::spawn(|| {
            let _ = Command::new("cargo")
                .args(&[
                    "embed",
                    "-p", "firmware",
                    "--target", "thumbv7em-none-eabihf",
                    "--bin", "micro",
                ])
                .output();
        });

        // Immediately send a response back to the client
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

  */





    let (status_line, filename, content_type) = if request_line.starts_with("GET / ") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/html/home.html", "text/html")
    } else if request_line.starts_with("GET /header.html") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/partials/header.html", "text/html")
    } else if request_line.starts_with("GET /footer.html") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/partials/footer.html", "text/html")
    } else if request_line.starts_with("GET /css/header.css") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/css/header.css", "text/css")
    } else if request_line.starts_with("GET /css/footer.css") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/css/footer.css", "text/css")
    } else if request_line.starts_with("GET /css/home.css") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/css/home.css", "text/css")
    } else if request_line.starts_with("GET /images/snakeheader.png") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/images/snakeheader.png", "image/png")
    } else if request_line.starts_with("GET /images/background.png") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/images/background.png", "image/png")
    } else if request_line.starts_with("GET /images/Cover.png") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/images/Cover.png", "image/png")
    }else if request_line.starts_with("GET /sim.html") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/html/sim.html", "text/html")
    } else if request_line.starts_with("GET /firmware.html") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/views/html/firmware.html", "text/html") }
    else if request_line.starts_with("GET /images/placeholder.png") {
        ("HTTP/1.1 200 OK", "../snakemicrobit/src/public/images/placeholder.png", "image/png")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "../micro/src/views/html/404.html", "text/html")
    };

    let path = Path::new(filename);

    if path.exists() && path.is_file() {
        if content_type == "image/png" {
            let contents = fs::read(filename).unwrap_or_else(|_| vec![]);
            let response = format!(
                "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                status_line,
                contents.len(),
                content_type
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&contents).unwrap();
        } else {
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
        let not_found = "<h1>404 - File Not Found</h1>".to_string();
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
            not_found.len(),
            not_found
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}

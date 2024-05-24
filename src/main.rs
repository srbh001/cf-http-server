// Uncomment this block to pass the first stage

#![allow(dead_code)]
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{env, usize};

#[derive(Debug, Clone)]
enum Method {
    POST,
    GET,
}

#[derive(Clone)]
struct StreamHandler {
    method: Method,
    path: String,
    path_param: String,
    content_length: usize,
    content_type: String,
    user_agent: String,
    data: String,
    encoding: String,
}

impl StreamHandler {
    fn print_request(stream_handler: StreamHandler) {
        println!(
            "[INFO] STREAM HANDLER RESPONSE 
    ===============================
    [INFO] Method : {:#?}
    [INFO] Path: {} 
    [INFO] Path Param: {}
    [INFO] Content-Encoding : {}
    [INFO] Content-Length: {}
    [INFO] User-Agent: {} 
    [INFO] Content-Type: {} 
    [INFO] Data: {} 
    [INFO] END OF SH RESPONSE
    ==============================
        ",
            stream_handler.method,
            stream_handler.path,
            stream_handler.path_param,
            stream_handler.encoding,
            stream_handler.content_length,
            stream_handler.user_agent,
            stream_handler.content_type,
            stream_handler.data
        );
    }
}

fn handle_stream(stream_buf: String) -> StreamHandler {
    let accepted_encodings = [
        "gzip",
        "deflate",
        "compress",
        "br",
        "zstd",
        "exi",
        "identity",
        "pack200-gzip",
    ];

    let blank_string = String::from("");
    let mut line_iterator = stream_buf.lines();
    let mut stream_handler: StreamHandler = StreamHandler {
        method: Method::GET,
        path: String::from("/"),
        path_param: String::from(""),
        content_length: 0,
        content_type: blank_string.clone(),
        user_agent: blank_string.clone(),
        data: blank_string.clone(),
        encoding: blank_string.clone(),
    };

    while let Some(line_content) = line_iterator.next() {
        if line_content.starts_with("GET /") {
            stream_handler.method = Method::GET; // Assuming some type of HTTP request only.
            let mut line_content_iter = line_content.split_whitespace();
            line_content_iter.next(); // skip the "GET"
            match line_content_iter.next() {
                Some(full_path) => {
                    let mut path_iter = full_path.split("/");
                    path_iter.next();
                    stream_handler.path = String::from("/") + path_iter.next().unwrap_or("");
                    stream_handler.path_param = String::from(path_iter.next().unwrap_or(""));
                }
                None => {
                    stream_handler.path = String::from("/");
                    stream_handler.path_param = String::from("");
                }
            }
        } else if line_content.starts_with("POST /") {
            stream_handler.method = Method::POST; // Assuming some type of HTTP request only.
            let mut line_content_iter = line_content.split_whitespace();
            line_content_iter.next(); // skip the "GET"
            match line_content_iter.next() {
                Some(full_path) => {
                    full_path.replace(" ", "");
                    let mut path_iter = full_path.split("/");
                    path_iter.next(); // Skip the blank char
                    stream_handler.path = String::from("/") + path_iter.next().unwrap_or("");
                    stream_handler.path_param = String::from(path_iter.next().unwrap_or(""));
                }
                None => {
                    stream_handler.path = String::from("/");
                    stream_handler.path_param = String::from("");
                }
            }
        }
        if line_content.starts_with("Content-Type") {
            let mut line_content_iter = line_content.split_whitespace();
            line_content_iter.next(); // skip to content type
            match line_content_iter.next() {
                Some(content_type) => {
                    stream_handler.content_type = String::from(content_type);
                }
                None => {
                    stream_handler.content_type = String::from("");
                }
            }
        }
        if line_content.starts_with("Content-Length") {
            let mut line_content_iter = line_content.split_whitespace();
            line_content_iter.next();
            match line_content_iter.next() {
                Some(content_length) => {
                    stream_handler.content_length = content_length.parse::<usize>().unwrap_or(0);
                }
                None => stream_handler.content_length = 0,
            }
        }
        if line_content.starts_with("User-Agent") {
            let user_agent = line_content.replace("User-Agent: ", "");

            stream_handler.user_agent = user_agent;
        }
        if line_content.starts_with("Accept-Encoding: ") {
            stream_handler.encoding = String::from("");
            let encodings = line_content
                .replace("Accept-Encoding: ", "")
                .replace(" ", "");
            let mut encoding_iter = encodings.split(",");

            while let Some(_encoding) = encoding_iter.next() {
                let encoding = String::from(_encoding);
                if accepted_encodings.contains(&encoding.as_str()) {
                    stream_handler.encoding = encoding;
                    break;
                }
            }
        }
        if line_content.is_empty() {
            let mut line_iterator_cloned = line_iterator.clone();

            while let Some(body_content) = line_iterator_cloned.next() {
                stream_handler.data += body_content;
            }
            break;
        }
    }
    stream_handler
}

fn handle_request(mut stream: TcpStream, directory: &str) {
    let ok_response = "HTTP/1.1 200 OK\r\n";
    let bad_request_response = "HTTP/1.1 400 Bad Request\r\n";
    let not_found_reponse = "HTTP/1.1 404 Not Found\r\n";
    let content_octet_header = "Content-Type: application/octet-stream\r\nContent-Length: ";
    let text_header = "Content-Type: text/plain\r\nContent-Length: ";
    let created_response = "HTTP/1.1 201 Created\r\n";

    let escape_chars = "\r\n\r\n";

    let mut encoding_header = String::from("");

    let mut buffer = [0; 2048];
    let _ = stream
        .peek(&mut buffer)
        .expect("[ERROR] Error reading from the stream");
    let request = String::from_utf8_lossy(&buffer);
    let stream_handler = handle_stream(request.into_owned());

    let encoding = stream_handler.clone().encoding;
    if !encoding.is_empty() {
        encoding_header += format!("Content-Encoding: {}\r\n", encoding).as_str();
    }

    StreamHandler::print_request(stream_handler.clone());

    if matches!(stream_handler.method, Method::GET) {
        if stream_handler.path == "/" {
            let content_to_send = format!("{ok_response}{encoding_header}{text_header}0\r\n\r\n");
            stream.write_all(content_to_send.as_bytes()).unwrap();
        } else if stream_handler.path == "/echo" {
            if !stream_handler.encoding.is_empty() {
                let path_param = stream_handler.path_param.as_bytes();
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&path_param).unwrap();
                let content_to_send_u = encoder.finish().unwrap();

                let content_length = content_to_send_u.len();
                let content = format!(
                    "{ok_response}{encoding_header}{text_header}{content_length}{escape_chars}",
                );

                let header_bytes = content.as_bytes();
                let mut final_content = Vec::new();
                final_content.extend_from_slice(header_bytes);
                final_content.extend_from_slice(&content_to_send_u);

                stream.write_all(&final_content).unwrap();
            //stream.write_all(&content_to_send_u).unwrap();
            } else {
                let body = stream_handler.path_param;

                let content_length = body.len();

                let content = format!(
                    "{ok_response}{encoding_header}{text_header}{content_length}{escape_chars}{body}",
                );
                stream.write_all(content.as_bytes()).unwrap();
            }
        } else if stream_handler.path == "/user-agent" {
            let content_length = stream_handler.user_agent.len();
            let user_agent = stream_handler.user_agent;
            let content = format!("{ok_response}{encoding_header}{text_header}{content_length}{escape_chars}{user_agent}");
            stream.write_all(content.as_bytes()).unwrap();
        } else if stream_handler.path == "/files" {
            let mut dir_str = String::from(directory);
            let file_name = stream_handler.path_param;
            if !dir_str.ends_with("/") {
                dir_str += "/";
            }
            let file_path = format!("{dir_str}{file_name}");
            let file_path_buf = PathBuf::from(file_path);
            let file = File::open(file_path_buf);
            match file {
                Ok(mut _file) => {
                    let mut file_content = String::new();
                    _file.read_to_string(&mut file_content).unwrap();
                    println!("[INFO] File Contents {}", file_content);
                    let content_length = file_content.len();
                    let data_to_send = format!(
                        "{ok_response}{encoding_header}{content_octet_header}{content_length}\r\n\r\n{file_content}"
                    );
                    stream.write_all(data_to_send.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
                Err(err) => {
                    let err_msg = String::from("[ERROR] : File Not Found");
                    let content_length = err_msg.len();
                    let data_to_send = format!(
                        "{not_found_reponse}{encoding_header}{text_header}{content_length}{escape_chars}{err_msg}"
                    );
                    stream.write_all(data_to_send.as_bytes()).unwrap();
                    eprintln!("[ERROR] File Not found {}", err);
                }
            }
        } else {
            let content_to_send =
                format!("{}{}{}{}", not_found_reponse, text_header, 0, escape_chars);
            stream.write_all(content_to_send.as_bytes()).unwrap();
        }
    } else if matches!(stream_handler.method, Method::POST) {
        if stream_handler.path == "/files" {
            let file_name = stream_handler.path_param;
            let mut dir_str = String::from(directory);
            if !dir_str.ends_with("/") {
                dir_str += "/";
            }
            let full_path = dir_str + file_name.as_str();
            let new_file_path = PathBuf::from(full_path);
            println!("[INFO] : New File Path - {}", new_file_path.display());

            match File::create(&new_file_path) {
                Ok(mut new_file) => {
                    let content_length = stream_handler.content_length;
                    let mut content = stream_handler.data;
                    content.truncate(content_length);

                    if content.len() == content_length {
                        if new_file.write_all(content.as_bytes()).is_ok() {
                            let c_r =
                                format!("{created_response}{encoding_header}{content_octet_header}{content_length}\r\n\r\n{content}");

                            stream.write_all(c_r.as_bytes()).unwrap();
                        } else {
                            eprintln!("[ERROR] : Failed to write to file");
                            stream.write_all(bad_request_response.as_bytes()).unwrap();
                        }
                    } else {
                        eprintln!("[ERROR] : Content length mismatch");
                        if new_file.write_all(content.as_bytes()).is_ok() {
                            println!("[INFO] : Content {}", content);
                            let c_r =
                                format!("{created_response}{encoding_header}{content_octet_header}{content_length}\r\n\r\n{content}");
                            stream.write_all(c_r.as_bytes()).unwrap();
                            println!("[INFO] : Still Wrote to file {}", content);
                        } else {
                            eprintln!("[ERROR] : Failed to write to file");
                            stream.write_all(bad_request_response.as_bytes()).unwrap();
                        }
                    }
                }

                Err(err) => {
                    eprintln!("[ERROR] : While creating the file: {}", err);
                    stream.write_all(bad_request_response.as_bytes()).unwrap();
                }
            }
        } else {
            println!(
                "[ERROR] Path Not Yet Implemented yet {}{}",
                stream_handler.path, stream_handler.path_param
            );
            let content_to_send = format!(
                "{}{}{}{}{}",
                not_found_reponse, encoding_header, text_header, 0, escape_chars
            );
            stream.write_all(content_to_send.as_bytes()).unwrap();
        }
    }
}

fn main() {
    println!("[INFO] : Logs from your program will appear here!");

    let arguments: Vec<String> = env::args().collect();
    println!("{:?}", arguments);
    let mut directory = String::from("");
    if arguments.len() >= 2 && arguments[1] == "--directory" {
        directory = arguments[2].clone();
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("[INFO]: New Connection");
                let stream = _stream;
                //handle_client(stream, directory.as_str());
                handle_request(stream, directory.as_str());
            }

            Err(e) => {
                println!("[ERROR] : While listening to stream {}", e);
            }
        }
    }
}

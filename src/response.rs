use std::io::Write;
use std::net::{TcpStream, Shutdown};

pub fn bad_request(stream: &mut TcpStream)                  { respond_4xx(stream, b"HTTP/1.0 400 Bad Request\r\n\r\n") }
pub fn not_found(stream: &mut TcpStream)                    { respond_4xx(stream, b"HTTP/1.0 404 Not Found\r\n\r\n") }
pub fn bad_method(stream: &mut TcpStream)                   { respond_4xx(stream, b"HTTP/1.0 405 Method Not Allowed\r\n\r\n") }
pub fn request_too_large(stream: &mut TcpStream)            { respond_4xx(stream, b"HTTP/1.0 413 Request Too Large\r\n\r\n") }

pub fn http_version_not_supported(stream: &mut TcpStream)   { respond_5xx(stream, b"HTTP/1.0 505 HTTP Version Not Supported\r\n\r\n") }
pub fn internal_server_error(stream: &mut TcpStream)        { respond_5xx(stream, b"HTTP/1.0 500 Internal Server Error\r\n\r\n") }

fn respond_4xx(stream: &mut TcpStream, error: &[u8]) {
    debug_assert!(error.starts_with(b"HTTP/1.0 4"));
    debug_assert!(error.ends_with(b"\r\n\r\n"));
    if stream.write_all(error).is_err() { return }
    if stream.shutdown(Shutdown::Both).is_err() { return }
}

fn respond_5xx(stream: &mut TcpStream, error: &[u8]) {
    debug_assert!(error.starts_with(b"HTTP/1.0 5"));
    debug_assert!(error.ends_with(b"\r\n\r\n"));
    if stream.write_all(error).is_err() { return }
    if stream.shutdown(Shutdown::Both).is_err() { return }
}

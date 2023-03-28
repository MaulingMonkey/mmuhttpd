use crate::*;

use std::io::{Read, Write};
use std::net::*;



pub fn run() {
    let settings = &*Box::leak(Box::new(Settings::from_env_or_die()));

    let mut port = 9000;
    let listener = loop {
        port += 1;
        assert!(port <= 9999, "cannot open a TcpListener on any port between 9000 ..= 9999");
        if let Ok(l) = TcpListener::bind((settings.bind, port)) { break l }
    };
    let url = format!("http://{}/", SocketAddr::from((
        if !settings.bind.is_unspecified()  { settings.bind }
        else if settings.bind.is_ipv6()     { IpAddr::V6(Ipv6Addr::LOCALHOST) }
        else                                { IpAddr::V4(Ipv4Addr::LOCALHOST) }
    , port)));

    println!("listening on {}, open {url} to view", SocketAddr::from((settings.bind, port)));
    if settings.open { browser::open_url(&url); }

    for connection in listener.incoming() {
        let connection = connection.expect("unable to accept incoming connection");
        let _ = std::thread::spawn(move || on_connection(settings, connection));
    }
}

fn on_connection(settings: &Settings, mut stream: TcpStream) {
    let mut header = [0u8; 8 * 1024]; // common limit per https://stackoverflow.com/a/60623751/953531
    let header = match read_header_discard_body(&mut stream, &mut header[..]) { Err(()) => return, Ok(h) => h };
    let (request, _headers) = header.split_once(b"\r\n").unwrap_or((header, b""));
    let Some((method, after_method)) = request.split_once(b" ") else { return response::bad_request(&mut stream) };
    let Some((path_search, version)) = after_method.split_once(b" ") else { return response::bad_request(&mut stream) };
    if ![b"HTTP/1.0", b"HTTP/1.1"].iter().any(|v| &v[..] == version) { return response::http_version_not_supported(&mut stream) };
    let (mut path, search) = path_search.split_at(path_search.find_window(b"?").unwrap_or(path_search.len()));
    debug_assert!(search.is_empty() || search.starts_with(b"?"));

    if !path.starts_with(b"/") { return response::not_found(&mut stream) }

    // TODO: escape hatches for magic paths (keepalive requests?)

    if path.find_window(b"//").is_some() { return response::not_found(&mut stream) } // XXX: excessive validation?
    if path.find_window(b"\\").is_some() { return response::not_found(&mut stream) } // XXX: excessive validation?
    let is_dir = path.ends_with(b"/");
    while let &[b'/', ref rest @ ..] = path { path = &rest }
    while let &[ref rest @ .., b'/'] = path { path = &rest }

    // XXX: this is a half-baked safety feature: by enumerating the filesystem for existing paths instead of directly
    // passing user-controlled paths to system APIs, we hopefully avoid allowing the user to (ab)use system specific
    // escape hatches like UNC paths, `CON` files, etc. - we also get consistent case sensitive paths on multiple OSes
    // regardless of the case sensitivity of the underlying OS or filesystem.
    //
    // This only really helps us out because we're providing a read-only abstraction.  Well, writes would be okay too,
    // but *creating* files with user controlled names wouldn't work with this trick.
    let Some(mut snapshot) = settings.cache.read_dir(&settings.root) else { return response::internal_server_error(&mut stream) };
    let mut file = b"index.html".as_slice();
    if !path.is_empty() {
        let mut dirs = path.split(|ch| *ch == b'/');
        if dirs.clone().any(|dir| dir.is_empty() || dir.starts_with(b".")) { return response::not_found(&mut stream) } // ban ".", "..", ".git", ".other_hidden_folder"
        if !is_dir { file = dirs.next_back().expect("bug: split should always return at least one element?"); }

        for dir in dirs {
            let dir = String::from_utf8_lossy(dir); // XXX: this is awkward, do this earlier
            let Some(entry) = snapshot.by_name(&*dir) else { return response::not_found(&mut stream) };
            let Some(next_snapshot) = settings.cache.read_dir(entry.path()) else { return response::not_found(&mut stream) };
            snapshot = next_snapshot;
        }
    }

    let Some(file_entry) = snapshot.by_name(&*String::from_utf8_lossy(file)) else { return response::not_found(&mut stream) };
    let Ok(file) = std::fs::File::open(file_entry.path()) else { return response::not_found(&mut stream) };
    let Ok(meta) = file.metadata() else { return response::internal_server_error(&mut stream) };
    let len = meta.len();
    let mime = mime::by_path(file_entry.name_lossy());
    let Some(mime) = mime else { return response::not_found(&mut stream) }; // ban access anything without a mime
    let headers = format!("HTTP/1.0 200 OK\r\nContent-Length: {len}\r\nContent-Type: {mime}\r\n\r\n");

    match method {
        b"HEAD" => {
            if stream.write_all(headers.as_bytes()).is_err() { return }
        },
        b"GET" => {
            let mut file = std::io::BufReader::new(file.take(len));
            if stream.write_all(headers.as_bytes()).is_err() { return }
            if std::io::copy(&mut file, &mut stream).is_err() { return }
        },
        _ => return response::bad_method(&mut stream),
    }
    if stream.shutdown(Shutdown::Both).is_err() { return }
}

/// N.B. this "discards" anything after the crlfcrlf (aka request body contents)
fn read_header_discard_body<'h>(stream: &mut TcpStream, header: &'h mut [u8]) -> Result<&'h [u8], ()> {
    debug_assert!(header.len() > 4);
    let mut len = 0;
    while len < header.len() {
        match stream.read(&mut header[len..]) {
            Err(_io)    => return Err(response::request_too_large(stream)),
            Ok(0)       => return Err(response::bad_request(stream)),
            Ok(read) => {
                let crlfcrlf = b"\r\n\r\n"; // marks end of HTTP request headers
                let crlfcrlf_search_start = len.saturating_sub(crlfcrlf.len()-1);
                len += read;
                for (offset, window) in header[crlfcrlf_search_start .. len].windows(crlfcrlf.len()).enumerate() {
                    if window == crlfcrlf {
                        return Ok(&header[.. crlfcrlf_search_start + offset]);
                    }
                }
            },
        }
    }
    Err(())
}

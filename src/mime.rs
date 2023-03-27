use std::ffi::OsStr;
use std::path::*;

pub fn by_path(path: &(impl AsRef<Path> + ?Sized)) -> Option<&'static str> {
    let path = path.as_ref();
    let ext = path.extension()?;
    by_extension(ext)
}

pub fn by_extension(ext: &(impl AsRef<OsStr> + ?Sized)) -> Option<&'static str> {
    let ext = ext.as_ref();
    let ext = ext.to_str()?;
    let ext = if ext.starts_with(".") { &ext[1..] } else { ext };
    Some(match ext {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
        // https://www.iana.org/assignments/media-types/media-types.xhtml#audio

        "html" | "htm"  => "text/html",
        "css"           => "text/css",
        "mjs" | "js"    => "text/javascript",
        "csv"           => "text/csv",
        "txt"           => "text/plain",
        "json"          => "application/json",
        "xhtml"         => "application/xhtml+xml",
        "xml"           => "application/xml",
        "rss"           => "application/atom+xml",

        "bmp"           => "image/bmp",
        "gif"           => "image/gif",
        "ico"           => "image/vnd.microsoft.icon",
        "jpg" | "jpeg"  => "image/jpeg",
        "png"           => "image/png",
        "svg"           => "image/svg+xml",
        "tif" | "tiff"  => "image/tiff",

        "otf"           => "font/otf",
        "sfnt"          => "font/sfnt",
        "ttf"           => "font/ttf",
        "woff"          => "font/woff",
        "woff2"         => "font/woff2",

        "mid" | "midi"  => "audio/midi",
        "mp3"           => "audio/mpeg",
        "mp4"           => "video/mp4",
        "mpeg"          => "video/mpeg",
        "oga"           => "audio/ogg",
        "ogv"           => "video/ogg",
        "opus"          => "audio/opus",
        "wav"           => "audio/wav",
        "weba"          => "audio/webm",
        "webm"          => "video/webm",

        "jar"           => "application/java-archive",
        "pdf"           => "application/pdf",
        "wasm"          => "application/wasm",

        "7z"            => "application/x-7z-compressed",
        "bz"            => "application/x-bzip",
        "bz2"           => "application/x-bzip2",
        "gz"            => "application/gzip",
        "tar"           => "application/x-tar",
        "rar"           => "application/vnd.rar",
        "zip"           => "application/zip",

        "bin"           => "application/octet-stream", // or other misc. binary data

        _               => return None
    })
}

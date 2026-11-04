#![forbid(unsafe_op_in_unsafe_fn)]

mod browser;
mod ext_slice;  use ext_slice::*;
mod fs;
mod libc;
mod mime;
mod response;
mod run;
mod settings;   use settings::*;
mod webdav;

fn main() { run::run() }

#![forbid(unsafe_op_in_unsafe_fn)]

mod browser;
mod env;
mod slice;
mod fs;
mod libc;
mod mime;
#[path = "TcpListener2.rs"] mod tcp_listener_2; use tcp_listener_2::TcpListener2;
mod response;
mod run;
mod settings;   use settings::*;
mod webdav;

fn main() { run::run() }

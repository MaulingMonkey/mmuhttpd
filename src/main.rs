mod browser;
mod ext_slice;  use ext_slice::*;
mod fs;
mod mime;
mod response;
mod run;
mod settings;   use settings::*;

fn main() { run::run() }

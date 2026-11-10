use std::env;
use std::path::PathBuf;
use yerpc::{Message, typescript::export_types_to_file};

fn main() {
    let outpath: PathBuf = env::args().nth(1).expect("Outpath is required").into();
    export_types_to_file::<Message>(&outpath, None).expect("Failed to write TS out");
}

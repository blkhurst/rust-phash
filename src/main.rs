// Declare .rs files as the module crate::files
mod args;
mod types;

// Bring symbols from the module into scope
use crate::args::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();

    println!("{:#?}", args);
}

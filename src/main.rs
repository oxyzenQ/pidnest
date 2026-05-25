mod cli;
mod procfs;
mod render;
mod tree;
mod user;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    if let Err(error) = cli::run(args) {
        eprintln!("pidnest: {error}");
        std::process::exit(1);
    }
}

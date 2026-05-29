//! Talos CLI — primary command-line interface.

use clap::Parser;

#[derive(Parser)]
#[command(name = "talos", version, about = "Next-generation agent runtime")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}

use clap::{Parser, Subcommand};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::new();
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    sub: Sub,
}
#[derive(Subcommand)]
enum Sub {
    // sub command hear
    // #[clap(short, long)]
}
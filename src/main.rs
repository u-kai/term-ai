use clap::{Parser, Subcommand};
use term_ai::gpt::GptClient;

#[tokio::main]
async fn main() {
    let gpt = GptClient::from_env().unwrap();
    gpt.chat("k8sについて教えて").await.unwrap();
    gpt.chat("なんでそんなにいいの？").await.unwrap();
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

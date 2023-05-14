use clap::{Parser, Subcommand};
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

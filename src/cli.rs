use clap::{Parser, Subcommand};
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    sub: Sub,
}
//#[derive(Subcommand)]
//enum Sub {
//#[clap(short, long)]
//Gpt3,
//#[clap(short, long)]
//English
//}

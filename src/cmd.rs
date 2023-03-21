use clap::{Subcommand, Parser,Args};

#[derive(Parser)]
struct Cmd {
    #[clap(subcommand)]
    subcmd: Command,
}

#[derive(Subcommand)]
enum Command {
    Version,
    #[clap(subcommand)]
    Balance(CommandBalance),
    #[clap(subcommand)]
    TX(CommandTX),
}

#[derive(Subcommand)]
enum CommandBalance {
   List,
}

#[derive(Subcommand)]
enum CommandTX {
    Add(AddArgs),
}
#[derive(Args)]
struct AddArgs {
    #[clap(short, long)]
    from: String,
    #[clap(short, long)]
    to: String,
    #[clap(short, long)]
    value: u64,
}

fn main() {
    env_logger::init();
    let cmd = Cmd::parse();

    match cmd.subcmd {
        Command::Version => {
            println!("version 0.1.0");
        }
        Command::Balance(cmd) => {
            match cmd {
                CommandBalance::List => {
                    println!("list balances");
                }
            }
        }
        Command::TX(cmd) => {
            match cmd {
                CommandTX::Add(args) => {
                    println!("add tx from {} to {} value {}", args.from, args.to, args.value);
                }
            }
        }
    }
}
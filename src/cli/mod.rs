use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Sonar CLI")]
#[command(about = "A command-line interface for Sonar", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Transmit {
        #[arg(short, long)]
        message: String,
    },
    Listen,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transmit { message } => {
            println!("Transmitting: {}", message);
            // Call sonar transmission function here
        }
        Commands::Listen => {
            println!("Listening for signals...");
            // Call sonar listening function here
        }
    }
}

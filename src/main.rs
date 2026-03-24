use clap::Parser;
use terminal_ascii_art::cli::Cli;
use terminal_ascii_art::run;

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

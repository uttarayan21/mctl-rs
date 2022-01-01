mod cli;
mod config;
mod control;
mod error;
use control::{Operation, Player};
use error::Error;

fn main() -> Result<(), Error> {
    let config = config::Config::try_load().unwrap_or_default();
    let matches = cli::build_cli().get_matches();
    let operation: Operation = matches
        .value_of("operation")
        .and_then(|op| op.parse::<Operation>().ok())
        .unwrap(); // clap should already quit if the values don't match;

    let mut control = control::Control::with_config(config).unwrap();

    let player = if matches.is_present("player") {
        matches.value_of("player").unwrap().parse::<Player>()?
    } else {
        control.player()?
    };

    control.handle(operation, player)?;
    Ok(())
}

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
        .unwrap() // clap should already quit if the values don't match
        .parse::<Operation>()?;

    let mut control = control::Control::with_config(config).unwrap();

    let player: Player;
    if matches.is_present("player") {
        player = matches.value_of("player").unwrap().parse::<Player>()?;
    } else {
        player = control.player()?;
    }

    control.handle(operation, player)?;
    Ok(())
}

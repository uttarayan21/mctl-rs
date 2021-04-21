mod config;
mod control;
mod error;
use clap::{App, Arg};
use control::{Operation, Player};
use error::Error;

fn main() -> Result<(), Error> {
    let config = config::Config::try_load().unwrap_or_default();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("operation")
                // .index(1)
                .required(true)
                .possible_values(&["play", "pause", "toggle", "stop", "status"]),
        )
        .arg(
            Arg::with_name("player")
                .long("--player")
                .short("-p")
                .value_name("player")
                .required(false)
                .possible_values(&["both", "mpd", "mpris"]),
        )
        .get_matches();
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

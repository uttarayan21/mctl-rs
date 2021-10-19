use clap::{arg_enum, App, Arg};

arg_enum! {
    #[derive(PartialEq,Debug)]
    pub enum ArgOperation {
        Play,
        Pause,
        Toggle,
        Prev,
        Next,
        Stop,
        Status,
    }
}

arg_enum! {
    #[derive(PartialEq,Debug)]
    pub enum ArgPlayer {
        Mpris,
        Mpd,
        Both,
    }
}

pub fn build_cli() -> clap::App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("operation")
                .index(1)
                .help("Specify the operation")
                .required(true)
                .possible_values(&ArgOperation::variants())
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("player")
                .long("--player")
                .short("-p")
                .value_name("player")
                .help("Specify the player")
                .required(false)
                .possible_values(&ArgPlayer::variants())
                .case_insensitive(true),
        )
}

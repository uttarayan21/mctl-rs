use crate::cli;
use crate::config::Config;
use crate::error::{Error, ErrorKind};
use std::{convert::TryFrom, str::FromStr};
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub enum Player {
    MPD,
    MPRIS,
    Both,
    None,
}

impl FromStr for Player {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mpd" => Ok(Self::MPD),
            "mpris" => Ok(Self::MPRIS),
            "both" => Ok(Self::Both),
            _ => Ok(Self::None),
        }
    }
}

impl From<cli::ArgPlayer> for Player {
    fn from(player: cli::ArgPlayer) -> Self {
        match player {
            cli::ArgPlayer::Both => Self::Both,
            cli::ArgPlayer::MPRIS => Self::MPRIS,
            cli::ArgPlayer::MPD => Self::MPD,
        }
    }
}

#[derive(Debug)]
pub enum Operation {
    Play,
    Pause,
    Toggle,
    Prev,
    Next,
    Stop,
    Status,
}

impl FromStr for Operation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "play" => Ok(Self::Play),
            "pause" => Ok(Self::Pause),
            "toggle" => Ok(Self::Toggle),
            "prev" => Ok(Self::Prev),
            "next" => Ok(Self::Next),
            "stop" => Ok(Self::Stop),
            "status" => Ok(Self::Status),
            _ => Err(Error::new(ErrorKind::UnknownOperationError)),
        }
    }
}

impl From<cli::ArgOperation> for Operation {
    fn from(operation: cli::ArgOperation) -> Self {
        match operation {
            cli::ArgOperation::Play => Self::Play,
            cli::ArgOperation::Pause => Self::Pause,
            cli::ArgOperation::Toggle => Self::Toggle,
            cli::ArgOperation::Prev => Self::Prev,
            cli::ArgOperation::Next => Self::Next,
            cli::ArgOperation::Stop => Self::Stop,
            cli::ArgOperation::Status => Self::Status,
        }
    }
}

#[derive(Debug)]
pub enum State {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug)]
pub struct PlayerInfo {
    kind: Player,
    title: String,
    // state: State,
    artists: Vec<String>,
}

impl TryFrom<&mut mpd::client::Client> for PlayerInfo {
    type Error = Error;
    fn try_from(mut client: &mut mpd::client::Client) -> Result<Self, Error> {
        let title = client
            .currentsong()?
            .unwrap_or_default()
            .title
            .unwrap_or_default()
            .to_string();
        let artists = Vec::new();
        Ok(Self {
            kind: Player::MPD,
            title,
            artists,
        })
    }
}

impl TryFrom<&mut mpris::Player<'_>> for PlayerInfo {
    type Error = Error;
    fn try_from(mut client: &mut mpris::Player) -> Result<Self, Self::Error> {
        let metadata = client.get_metadata().unwrap();
        let title = metadata.title().unwrap_or_default().to_string();
        let artists = metadata.artists().unwrap_or(&Vec::new()).to_vec();
        Ok(Self {
            kind: Player::MPRIS,
            title,
            artists,
        })
    }
}

#[derive(Debug)]
pub struct Control<'c> {
    mpdclient: Option<mpd::Client>,
    mprisclient: Option<mpris::Player<'c>>,
    priority: Player,
}

impl<'c> Control<'c> {
    pub fn with_config(config: Config) -> Result<Self, Error> {
        let mpdclient =
            mpd::Client::connect(format!("{}:{}", config.mpd_host, config.mpd_port)).ok();
        let mprisclient = mpris::PlayerFinder::new()?.find_active().ok();
        Ok(Self {
            mpdclient,
            mprisclient,
            priority: config.priority,
        })
    }
    pub fn player(&mut self) -> Result<Player, Error> {
        let mpdstatus = match &mut self.mpdclient {
            Some(c) => c.status()?.state,
            None => mpd::State::Stop,
        };
        let mprisstatus = match &self.mprisclient {
            Some(c) => c.is_running(),
            None => false,
        };
        let running: Player = match (mpdstatus, mprisstatus) {
            (mpd::status::State::Play, true) => Player::Both,
            (mpd::status::State::Play, _) => Player::MPD, // since both case is already covered first this should not include the both case.
            (_, true) => Player::MPRIS,
            (_, _) => Player::None,
        };
        Ok(running)
    }
    pub fn play(&mut self, player: Player) -> Result<(), Error> {
        match player {
            Player::MPD => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.play()?;
                }
            }
            Player::MPRIS => {
                if let Some(mprisclient) = &mut self.mprisclient {
                    mprisclient.play()?;
                }
            }
            Player::Both => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.play()?;
                }
                if let Some(mprisclient) = &mut self.mprisclient {
                    mprisclient.play()?;
                }
            }
            _ => (),
        };
        Ok(())
    }
    pub fn pause(&mut self) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            mpdclient.pause(true)?;
        }
        if let Some(mprisclient) = &mut self.mprisclient {
            mprisclient.pause()?;
        }
        Ok(())
    }
    pub fn toggle(&mut self) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            mpdclient.toggle_pause()?;
        }
        if let Some(mprisclient) = &mut self.mprisclient {
            mprisclient.play_pause()?;
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<(), Error> {
        Ok(())
    }
    pub fn prev(&mut self) -> Result<(), Error> {
        Ok(())
    }
    pub fn stop(&mut self) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            mpdclient.stop()?;
        }
        if let Some(mprisclient) = &mut self.mprisclient {
            mprisclient.stop()?;
        }
        Ok(())
    }
    pub fn status(&mut self, player: Player) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            println!("{:?}", PlayerInfo::try_from(mpdclient));
        }
        if let Some(mprisclient) = &mut self.mprisclient {
            println!("{:?}", PlayerInfo::try_from(mprisclient));
        }
        Ok(())
    }
    pub fn handle(&mut self, operation: Operation, player: Player) -> Result<(), Error> {
        // let player = self.status()?;
        println!("{:?}", player);
        match operation {
            Operation::Play => Ok(self.play(player)?),
            Operation::Pause => Ok(self.pause()?),
            Operation::Toggle => Ok(self.toggle()?),
            Operation::Prev => Ok(self.prev()?),
            Operation::Next => Ok(self.next()?),
            Operation::Stop => Ok(self.stop()?),
            Operation::Status => Ok(self.status(player)?),
            // _ => Ok(()),
        }
    }
}

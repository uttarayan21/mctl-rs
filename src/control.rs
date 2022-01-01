use crate::cli;
use crate::config::Config;
use crate::error::{Error, ErrorKind};
use derive_more::Display;
use std::{convert::TryFrom, str::FromStr};
#[derive(Debug, Clone, Copy, Display, serde::Deserialize)]
pub enum Player {
    Mpd,
    Mpris,
    Both,
    None,
}

impl FromStr for Player {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mpd" => Ok(Self::Mpd),
            "mpris" => Ok(Self::Mpris),
            "both" => Ok(Self::Both),
            _ => Ok(Self::None),
        }
    }
}

impl From<cli::ArgPlayer> for Player {
    fn from(player: cli::ArgPlayer) -> Self {
        match player {
            cli::ArgPlayer::Both => Self::Both,
            cli::ArgPlayer::Mpris => Self::Mpris,
            cli::ArgPlayer::Mpd => Self::Mpd,
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

impl Default for Operation {
    fn default() -> Self {
        Self::Status
    }
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
            _ => Err(Error::new(ErrorKind::UnknownOperation)),
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

#[derive(Debug, Display)]
pub enum State {
    Playing,
    Paused,
    Stopped,
}

use mpris::PlaybackStatus;
impl From<PlaybackStatus> for State {
    fn from(playback_status: mpris::PlaybackStatus) -> State {
        match playback_status {
            PlaybackStatus::Paused => State::Paused,
            PlaybackStatus::Playing => State::Playing,
            PlaybackStatus::Stopped => State::Stopped,
        }
    }
}

use mpd::State as MPDState;
impl From<MPDState> for State {
    fn from(playback_status: mpd::State) -> State {
        match playback_status {
            MPDState::Stop => State::Stopped,
            MPDState::Pause => State::Paused,
            MPDState::Play => State::Playing,
        }
    }
}

#[derive(Debug)]
pub struct PlayerInfo {
    kind: Player,
    title: String,
    artists: Vec<String>,
    state: State,
}

impl std::fmt::Display for PlayerInfo {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Player: {}", self.kind)?;
        writeln!(f, "State: {}", self.state)?;
        writeln!(f, "Title: {}", self.title)?;
        if self.artists.len() > 1 {
            write!(f, "Artists: ")?;
        } else {
            write!(f, "Artist: ")?;
        }
        for (count, artist) in self.artists.iter().enumerate() {
            if count != 0 {
                write!(f, ", ")?
            }
            write!(f, "{}", artist)?;
        }
        Ok(())
    }
}

impl TryFrom<&mut mpd::client::Client> for PlayerInfo {
    type Error = Error;
    fn try_from(client: &mut mpd::client::Client) -> Result<Self, Error> {
        let title = client
            .currentsong()?
            .unwrap_or_default()
            .title
            .unwrap_or_default();

        let artists = Vec::new();
        let state = match client.status().unwrap_or_default().state {
            MPDState::Play => State::Playing,
            MPDState::Pause => State::Paused,
            MPDState::Stop => State::Stopped,
        };

        Ok(Self {
            kind: Player::Mpd,
            title,
            artists,
            state,
        })
    }
}

impl TryFrom<&mut mpris::Player<'_>> for PlayerInfo {
    type Error = Error;
    fn try_from(client: &mut mpris::Player) -> Result<Self, Self::Error> {
        let metadata = client.get_metadata().unwrap();
        let title = metadata.title().unwrap_or_default().to_string();
        let artists = metadata.artists().unwrap_or(&Vec::new()).to_vec();
        let state = State::from(client.get_playback_status().unwrap());
        Ok(Self {
            kind: Player::Mpris,
            title,
            artists,
            state,
        })
    }
}

#[allow(dead_code)]
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
            (mpd::status::State::Play, false) => Player::Mpd,
            (_, true) => Player::Mpris,
            (_, false) => Player::None,
        };
        Ok(running)
    }

    pub fn play(&mut self, player: Player) -> Result<(), Error> {
        match player {
            Player::Mpd => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.play()?;
                }
            }
            Player::Mpris => {
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

    pub fn next(&mut self, player: Player) -> Result<(), Error> {
        match player {
            Player::Mpd => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.next()?;
                }
            }
            Player::Mpris => {
                if let Some(mpristclient) = &mut self.mprisclient {
                    mpristclient.next()?;
                }
            }
            Player::Both => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.next()?;
                }
                if let Some(mpristclient) = &mut self.mprisclient {
                    mpristclient.next()?;
                }
            }
            _ => (),
        }
        Ok(())
    }

    pub fn prev(&mut self, player: Player) -> Result<(), Error> {
        match player {
            Player::Mpd => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.prev()?;
                }
            }
            Player::Mpris => {
                if let Some(mpristclient) = &mut self.mprisclient {
                    mpristclient.previous()?;
                }
            }
            Player::Both => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.prev()?;
                }
                if let Some(mpristclient) = &mut self.mprisclient {
                    mpristclient.previous()?;
                }
            }
            _ => (),
        }
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
        match player {
            Player::Mpd => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    println!("{}", PlayerInfo::try_from(mpdclient)?);
                }
            }
            Player::Mpris => {
                if let Some(mprisclient) = &mut self.mprisclient {
                    println!("{}", PlayerInfo::try_from(mprisclient)?);
                }
            }
            Player::Both => {
                if let Some(mprisclient) = &mut self.mprisclient {
                    println!("{}", PlayerInfo::try_from(mprisclient)?);
                }
                if let Some(mpdclient) = &mut self.mpdclient {
                    println!("{}", PlayerInfo::try_from(mpdclient)?);
                }
            }
            _ => (),
        }
        Ok(())
    }

    pub fn handle(&mut self, operation: Operation, player: Player) -> Result<(), Error> {
        println!("{:?}", player);
        match operation {
            Operation::Play => Ok(self.play(player)?),
            Operation::Pause => Ok(self.pause()?),
            Operation::Toggle => Ok(self.toggle()?),
            Operation::Prev => Ok(self.prev(player)?),
            Operation::Next => Ok(self.next(player)?),
            Operation::Stop => Ok(self.stop()?),
            Operation::Status => Ok(self.status(player)?),
        }
    }
}

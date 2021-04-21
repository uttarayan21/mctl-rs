// use mpd::playlist;

use crate::config::Config;
use crate::error::{Error, ErrorKind};
use std::str::FromStr;
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

#[derive(Debug)]
pub enum Operation {
    Play,
    Pause,
    Toggle,
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
            "stop" => Ok(Self::Stop),
            "status" => Ok(Self::Status),
            _ => Err(Error::new(ErrorKind::UnknownOperationError)),
        }
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
                if let Some(mprisclient) = &mut self.mpdclient {
                    mprisclient.play()?;
                }
            }
            Player::Both => {
                if let Some(mpdclient) = &mut self.mpdclient {
                    mpdclient.play()?;
                }
                if let Some(mprisclient) = &mut self.mpdclient {
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
        Ok(())
    }
    pub fn toggle(&mut self) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            mpdclient.toggle_pause()?;
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
    pub fn status(&mut self) -> Result<(), Error> {
        if let Some(mpdclient) = &mut self.mpdclient {
            println!("{:#?}", mpdclient.status()?);
        }
        if let Some(mprisclient) = &mut self.mprisclient {
            println!("{:#?}", mprisclient.get_metadata()?);
        }
        Ok(())
    }
    pub fn handle(&mut self, operation: Operation, player: Player) -> Result<(), Error> {
        // let player = self.status()?;
        match operation {
            Operation::Play => Ok(self.play(player)?),
            Operation::Pause => Ok(self.pause()?),
            Operation::Toggle => Ok(self.toggle()?),
            Operation::Stop => Ok(self.stop()?),
            Operation::Status => Ok(self.status()?),
        }
    }
}

use std::fmt;


pub struct Config {
    pub mode: Mode,
}

impl Config {
    pub fn from_args(mut args: std::env::Args) -> Result<Config, Error> {
        args.next();

        let mode_str = args.next().ok_or_else(Error::MissingModeArg)?;

        let mode = match &mode_str[..] {
            "find" => {
                let wanted_prefix = args.next().ok_or_else(Error::MissingPrefixArg)?;
                err_if_false(is_all_hex(&wanted_prefix), Error::NonHexPrefix())?;
                Ok(Mode::Find(wanted_prefix))
            },

            "update" => {
                let wanted_prefix = args.next().ok_or_else(Error::MissingPrefixArg)?;
                err_if_false(is_all_hex(&wanted_prefix), Error::NonHexPrefix())?;
                Ok(Mode::Update(wanted_prefix))
            },

            "revert" =>
                Ok(Mode::Revert()),

            _ =>
                Err(Error::UnsupportedMode()),
        }?;

        Ok(Config {
            mode,
        })
    }
}



pub enum Mode {
    Find(String),
    Update(String),
    Revert(),
}


pub enum Error {
    MissingModeArg(),
    MissingPrefixArg(),
    UnsupportedMode(),
    NonHexPrefix(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingModeArg() => {
                write!(f, "<mode> argument is missing")
            }

            Error::MissingPrefixArg() => {
                write!(f, "<prefix> argument is missing")
            }

            Error::UnsupportedMode() => {
                write!(f, "Unsupported <mode> argument given")
            }

            Error::NonHexPrefix() => {
                write!(f, "<prefix> must be hexadecimal")
            }
        }
    }
}


fn is_all_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

fn err_if_false<E>(value: bool, err: E) -> Result<(), E> {
    if value {
        Ok(())
    } else {
        Err(err)
    }
}

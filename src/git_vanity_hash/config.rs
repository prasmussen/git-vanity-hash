

pub struct Config {
    pub mode: Mode,
}

impl Config {
    pub fn from_args(mut args: std::env::Args) -> Option<Config> {
        args.next();

        let mode_str = args.next()?;

        let mode = match &mode_str[..] {
            "find" => {
                let wanted_prefix = args.next()?;
                Some(Mode::Find(wanted_prefix))
            },

            "update" => {
                let wanted_prefix = args.next()?;
                Some(Mode::Update(wanted_prefix))
            },

            "revert" =>
                Some(Mode::Revert()),

            _ =>
                None,
        }?;

        Some(Config {
            mode,
        })
    }
}



pub enum Mode {
    Find(String),
    Update(String),
    Revert(),
}

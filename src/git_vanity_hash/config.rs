

pub struct Config {
    pub mode: Mode,
    pub wanted_prefix: String,
}

impl Config {
    pub fn from_args(mut args: std::env::Args) -> Option<Config> {
        args.next();

        let mode = args.next()
            .and_then(|str| Mode::from_str(&str))?;

        let wanted_prefix = args.next()?;

        Some(Config {
            mode,
            wanted_prefix,
        })
    }
}



pub enum Mode {
    Find(),
    Update(),
}

impl Mode {
    fn from_str(str: &str) -> Option<Mode> {
        match str {
            "find" =>
                Some(Mode::Find()),

            "update" =>
                Some(Mode::Update()),

            _ =>
                None,
        }
    }
}

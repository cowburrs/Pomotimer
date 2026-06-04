use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Amount of time to study.
    /// refer to https://docs.rs/humantime/latest/humantime/fn.parse_duration.html
    #[arg(default_value = "50m")]
    pub study: String,

    /// Amount of time to rest.
    /// refer to https://docs.rs/humantime/latest/humantime/fn.parse_duration.html
    #[arg(default_value = "10m")]
    pub rest: String,

    /// General name of action, e.g. maths, physics.
    #[arg(default_value = "", short, long)]
    pub name: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a simple Pomodoro timer.
    Run(RunArgs),
    /// Connect to a peer (not implemented)
    Connect {
        #[arg(default_value = "")]
        name: String,
    },
    /// Host a online Pomotimer (not implemented)
    Host {
        #[arg(default_value = "")]
        name: String,
    },
}

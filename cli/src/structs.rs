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

#[derive(clap::Args, Debug)]
pub struct HostArgs {
    #[arg(default_value = "")]
    pub peer: String,

    /// Room name to join.
    #[arg(default_value = "default")]
    pub room: String,
}

#[derive(clap::Args, Debug)]
pub struct JoinArgs {
    /// Room name to join.
    #[arg(default_value = "default")]
    pub room: String,

    /// Connect directly to peer without host
    #[arg(default_value = "", short, long)]
    pub peer: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a simple feature rich solo Pomodoro timer.
    Run(RunArgs),
    /// Connect to a peer (not implemented)
    Join(JoinArgs),
    /// Host a online Pomotimer (not implemented)
    Host(HostArgs),
    /// List Public Lobbies (not implemented)
    List,
    //TODO: add heatmap to stats
    /// Show study stats, try not to run too much! (Not implemented)
    Stats,
}

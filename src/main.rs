use clap::Parser;
use std::time::Duration;

mod structs;

fn set_status() -> Box<dyn Fn(discord_rich_presence::activity::Activity)> {
    use discord_rich_presence::{activity, activity::Assets, DiscordIpc, DiscordIpcClient};
    struct Discord(DiscordIpcClient);

    impl Drop for Discord {
        fn drop(&mut self) {
            self.0.close().unwrap();
        }
    }

    let mut client = Discord(DiscordIpcClient::new("1503690275783184454"));

    if client.0.connect().is_ok() {
        client
            .0
            .set_activity(
                activity::Activity::new()
                    .state("In between..")
                    .details("Studying currently.")
                    .name("Pomodoro.")
                    .assets(Assets::new().small_image("todo").small_text("idrk")),
            )
            .unwrap();
        let client = std::cell::RefCell::new(client);
        return Box::new(move |act| {
            client.borrow_mut().0.set_activity(act).unwrap();
        });
    }
    Box::new(|_| {})
}

mod run;
fn main() {
    use structs::Args;
    use structs::Commands;
    let _status = set_status();
    let args = Args::parse();
    match args.command {
        Commands::Run(runargs) => {
            run::run(runargs, _status);
        }
        _ => {
            println!("Sorry I haven't implemented this yet. Coming soon tho!!")
        }
    }
}

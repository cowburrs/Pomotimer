use clap::Parser;

use crate::host::host_iroh;

mod host;
mod structs;

fn set_status() -> Box<dyn Fn(discord_rich_presence::activity::Activity)> {
    use discord_rich_presence::{activity, activity::Assets, DiscordIpc, DiscordIpcClient};
    struct Discord {
        dc: DiscordIpcClient,
        connected: bool,
    }

    let mut client = Discord {
        dc: DiscordIpcClient::new("1503690275783184454"),
        connected: false,
    };

    if client.dc.connect().is_ok() {
        if client
            .dc
            .set_activity(
                activity::Activity::new()
                    .state("In between..")
                    .details("Studying currently.")
                    .name("Pomodoro.")
                    .assets(Assets::new().small_image("todo").small_text("idrk")),
            )
            .is_ok()
        {
            client.connected = true;
        } else {
            println!("warning: Discord rich presence not established.")
        }

        let client = std::cell::RefCell::new(client);

        return Box::new(move |act| {
            client.borrow_mut().dc.set_activity(act).unwrap();
        });
    } else {
        println!("tip: You can use discord rich presence with Pomotimer!");
    }

    impl Drop for Discord {
        fn drop(&mut self) {
            if self.dc.close().is_ok() {
                println!("success: Discord connection successfully closed.");
            } else {
                if self.connected {
                    println!("warning: Discord connection improperly closed.");
                }
            }
        }
    }

    Box::new(|_| {})
}

mod run;
#[tokio::main]
async fn main() {
    use structs::Args;
    use structs::Commands;
    let args = Args::parse();
    match args.command {
        Commands::Run(runargs) => {
            let _status = set_status();
            run::run(runargs, _status);
        }
        Commands::Host { .. } => {
            let _ = host_iroh().await;
        }
        // Commands::Connect { .. } => {}
        _ => {
            println!("Sorry I haven't implemented this yet. Coming soon tho!!")
        }
    }
}

use clap::Parser;
use notify_rust::Notification;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    println!("Hello, world!");
    Notification::new()
        .summary("Firefox News")
        .body("This will almost look like a real firefox notification.")
        .icon("firefox")
        .timeout(5000)
        .show()
        .expect("failed to send notification");
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}

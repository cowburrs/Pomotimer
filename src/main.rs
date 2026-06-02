use core::time;
use std::time::Duration;

use clap::{Parser, Subcommand};
use notify_rust::Notification;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a simple Pomodoro timer.
    Run {
        #[arg(default_value = "50m")]
        study: String,
        #[arg(default_value = "10m")]
        rest: String,
    },
    /// Connect to a peer (not implemented)
    Connect {
        #[arg(default_value = "")]
        name: String,
    },
}
fn play_finish() {
    use std::io::BufReader;
    use std::io::Cursor;
    let mut sink_handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    sink_handle.log_on_drop(false);
    let cursor = Cursor::new(include_bytes!("../assets/sfx/POMODORO-FINISH.wav").as_ref());
    let file = BufReader::new(cursor);
    let player = rodio::play(&sink_handle.mixer(), file).unwrap();
    player.set_volume(0.15);
    player.sleep_until_end();
}

fn play_end() {
    use std::io::BufReader;
    use std::io::Cursor;
    let mut sink_handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    sink_handle.log_on_drop(false);
    let cursor = Cursor::new(include_bytes!("../assets/sfx/POMODORO-BREAK-END.wav").as_ref());
    let file = BufReader::new(cursor);
    let player = rodio::play(&sink_handle.mixer(), file).unwrap();
    player.set_volume(0.2);
    player.sleep_until_end();
}
fn send_notification(summary: &str, body: &str) {
    Notification::new()
        .summary(summary)
        .body(body)
        .timeout(5000)
        .show()
        .expect("failed to send notification");
}
fn timer(secs: Duration) {
    use crossterm::{cursor, event, execute, terminal};
    use std::io::stdout;
    let mut i = 0.0;
    loop {
        if i > secs.as_secs_f32() {
            break;
        }
        i += 0.1;
        println!(
            "{}/{}",
            humantime::format_duration(Duration::from_secs(secs.as_secs() - i as u64)),
            humantime::format_duration(secs)
        );
        print_bar_percent({ i / (secs.as_secs() as f32) });
        std::thread::sleep(time::Duration::from_millis(100));
        // event::poll(std::time::Duration::from_millis(100)).unwrap();
        execute!(
            stdout(),
           cursor::MoveUp(2),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )
        .unwrap();
    }
}
fn print_bar(width: u16) {
    use crossterm::execute;
    use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
    use std::io::stdout;
    for i in 0..width.clamp(0, 75) {
        let g = 50 + (i / 2) as u8;
        let base_r = 255 - width as u8 - (i * 2) as u8;
        let r = if width < 10 {
            (base_r as f32 * (width as f32 / 10.0).powf(0.3)).max(0.0) as u8
        } else {
            base_r
        };
        execute!(
            stdout(),
            SetForegroundColor(Color::Rgb { r, g, b: 255 }),
            Print("█"),
            ResetColor
        )
        .unwrap();
    }
    for _ in 0..75 - width {
        execute!(
            stdout(),
            SetForegroundColor(Color::Rgb {
                r: 45,
                g: 45,
                b: 45
            }),
            Print("█"),
            ResetColor
        )
        .unwrap();
    }
    print!("{}", ResetColor);
}
fn print_bar_percent(percentage: f32) {
    let percentage = percentage.clamp(0.0, 1.0);
    print_bar((percentage * (75 as f32)) as u16);
    println!(" {}%", (percentage * 100.0).round())
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Run { study, rest } => {
            use humantime::parse_duration;
            timer(parse_duration(&study).unwrap()); // BUG: I need to error handle this
                                                    // but i don't care tbh
            send_notification(
                "Study Finished",
                &format!("Your study timer of {} is finished", study),
            );
            play_finish();
            timer(parse_duration(&rest).unwrap()); // TODO: I want more granularity. like
                                                   // seconds and stuff
            send_notification(
                "Break Finished",
                &format!("Your rest timer of {} is finished", rest),
            );
            play_end();
        }
        Commands::Connect { .. } => {
            println!("Sorry I haven't implemented this yet. Coming soon tho!!")
        }
    }
}

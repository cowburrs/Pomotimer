use crate::structs::RunArgs;
use chrono::{NaiveDate, NaiveTime, Timelike};
use n0_error::AnyError;
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Deserialize, Serialize, Debug)]
struct Log {
    #[serde(rename = "userId")]
    date: NaiveDate,
    timestamps: Vec<NaiveTime>,
    subject: String,
    study: String,
    rest: String,
}

fn play_finish() {
    use std::io::BufReader;
    use std::io::Cursor;
    let mut sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
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
    let mut sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
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
fn timer(secs: Duration, name: &str) {
    use crossterm::event::{Event, KeyCode, KeyModifiers};
    use crossterm::{cursor, event, execute, terminal};
    use std::io::stdout;
    use std::time::Instant;
    let start = Instant::now();

    terminal::enable_raw_mode().unwrap();

    let dt = if secs.as_millis() / 100 > 1000 {
        1000
    } else {
        (secs.as_millis() / 100) as u64
    };
    loop {
        let elapsed = start.elapsed();
        let i = elapsed.as_secs_f32();

        if elapsed >= secs {
            break;
        }
        print!(
            "doing {} - {}/{}",
            name,
            humantime::format_duration(Duration::from_secs(secs.as_secs() - i as u64)),
            humantime::format_duration(secs)
        );
        print!("\r\n");
        print_bar_percent(i / (secs.as_secs() as f32));

        if event::poll(Duration::from_millis(dt)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('q') => {
                        execute!(
                            stdout(),
                            cursor::MoveUp(2),
                            terminal::Clear(terminal::ClearType::FromCursorDown),
                        )
                        .unwrap();
                        break;
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        execute!(
                            stdout(),
                            cursor::MoveUp(2),
                            terminal::Clear(terminal::ClearType::FromCursorDown),
                        )
                        .unwrap();
                        terminal::disable_raw_mode().unwrap();
                        eprintln!("warning: Session not saved. Use 'q' to quit instead of Ctrl+C.");
                        std::process::exit(1);
                    }
                    _ => {}
                }
            }
        }

        execute!(
            stdout(),
            cursor::MoveUp(2),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )
        .unwrap();
    }

    terminal::disable_raw_mode().unwrap();
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
    print!(" {}%", (percentage * 100.0).round());
    print!("\r\n")
}
fn write_log(log: Log) -> Result<(), ()> {
    // TODO: Rewrite this so it doesnt just rewrite the entire fucking file
    // again and waste a bunch of disk usage
    use directories::BaseDirs;
    use serde_json::from_str;
    use std::fs;
    if let Some(dir) = BaseDirs::new() {
        let target = dir.data_local_dir().join("pomotimer/pomodoro_log.json");

        let json = fs::read_to_string(&target).unwrap_or("[]".to_string());
        let mut todos = serde_json::from_str::<Vec<Log>>(&json).map_err(|_| ())?;
        todos.push(log);
        fs::write(
            dir.data_local_dir().join("pomotimer/pomodoro_log.json"),
            serde_json::to_string(&todos).unwrap(),
        )
        .unwrap();
    }
    Ok(())
}

enum PomoType {
    Rest,
    Study,
}

fn pomo_activity(
    name: &String,
    time: Duration,
    pomotype: PomoType,
) -> discord_rich_presence::activity::Activity<'static> {
    use discord_rich_presence::activity::{
        Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    let dnow = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut act = Activity::new() // TODO: Implement secrets/party when i make multiplayer.
        .name("Pomodoro")
        .details_url("https://github.com/cowburrs/Pomotimer")
        .state(format!("Doing {}", name))
        .activity_type(ActivityType::Competing)
        .status_display_type(StatusDisplayType::Name)
        .timestamps(
            Timestamps::new()
                .start(dnow)
                .end(dnow + time.as_secs() as i64),
        )
        .buttons(vec![
            // Button::new("Source Code", "https://github.com/charatwukki/Pomotimer"),
            // Button::new("Visit Site", "https://example.com"),
        ]);
    match pomotype {
        PomoType::Rest => {
            act = act
                .details("Resting zzz...")
                .state_url(format!(
                    "https://cowburrs.github.io/Pomotimer/book/rest.html"
                ))
                .assets(
                    Assets::new()
                        .large_image("todo")
                        .large_text("Pomodoro Timer")
                        .small_image("rest")
                        .small_text("Resting"),
                );
        }
        PomoType::Study => {
            act = act
                .details("Working Hard!")
                .state_url(format!(
                    "https://cowburrs.github.io/Pomotimer/book/study.html"
                ))
                .assets(
                    Assets::new()
                        .large_image("todo")
                        .large_text("Pomodoro Timer")
                        .small_image("study")
                        .small_text("Studying"),
                );
        }
    };
    act
}

pub fn run(
    runargs: crate::structs::RunArgs,
    _status: Box<dyn Fn(discord_rich_presence::activity::Activity)>,
) {
    let RunArgs {
        study,
        rest,
        mut name,
    } = runargs;
    if name.is_empty() {
        name = "pomodoro".to_string();
    }
    use humantime::parse_duration;
    let now = chrono::Local::now();
    let mut timestamps: Vec<NaiveTime> = vec![];
    let studyduration = parse_duration(&study)
        .inspect_err(|_| {
            eprintln!("error: unable to parse study, defaulting to 50m");
        })
        .unwrap_or(Duration::from_mins(50));
    let restduration = parse_duration(&study)
        .inspect_err(|_| {
            eprintln!("error: unable to parse rest, defaulting to 10m");
        })
        .unwrap_or(Duration::from_mins(10));

    timestamps.push(chrono::Local::now().time().with_nanosecond(0).unwrap());
    _status(pomo_activity(&name, studyduration, PomoType::Study));
    timer(studyduration, &name);
    send_notification(
        "Study Finished",
        &format!("Your study timer of {} is finished", study),
    );
    play_finish();
    timestamps.push(chrono::Local::now().time().with_nanosecond(0).unwrap());

    _status(pomo_activity(&name, restduration, PomoType::Rest));
    timer(restduration, &name);
    timestamps.push(chrono::Local::now().time().with_nanosecond(0).unwrap());
    send_notification(
        "Break Finished",
        &format!("Your rest timer of {} is finished", rest),
    );
    play_end();

    let log = Log {
        date: now.date_naive(),
        timestamps,
        subject: name,
        study, // awesome how i can just not define it and it will work
        rest,
    };

    write_log(log);
}

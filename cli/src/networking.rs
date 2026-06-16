use std::{any::Any, io::Write, time::Duration};

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use humantime::parse_duration;
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointAddr, EndpointId, SecretKey};
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender},
    Gossip, TopicId,
};
use n0_error::{Result, StdResultExt};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::time::sleep;

use crate::structs::{HostArgs, JoinArgs};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    body: MessageBody,
    nonce: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
enum MessageBody {
    Message { text: Vec<String> },
    Pomodoro { study: Duration, rest: Duration },
    Stop,
}

impl Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string().into())
    }

    pub fn new(body: MessageBody) -> Self {
        Self {
            body,
            nonce: rand::random(), // I just wanted to write a comment that i think this is so
                                   // awesome you do this because gossip makes it so that the same
                                   // message can't be either sent or received twice.
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

fn secret_key_from_passphrase(passphrase: &str) -> SecretKey {
    let hash = Sha256::digest(passphrase.as_bytes());
    SecretKey::from_bytes(&hash.into())
}
fn get_secret(string: &str) -> SecretKey {
    secret_key_from_passphrase(&("pomotimer".to_owned() + string))
}
pub async fn create_sender_receiver(
    hostargs: &HostArgs,
) -> Result<(GossipSender, GossipReceiver, Router)> {
    let endpoint = Endpoint::builder(presets::N0).bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes([23u8; 32]);

    let (sender, receiver) = gossip
        .subscribe(topic_id, vec![get_secret(&hostargs.room).public()])
        .await?
        .split();
    Ok((sender, receiver, router))
}

pub async fn create_entry(hostargs: &HostArgs) -> Result<(GossipReceiver, Router)> {
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(get_secret(&hostargs.room))
        .bind()
        .await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes([23u8; 32]);
    let (_, receiver) = gossip.subscribe(topic_id, vec![]).await?.split();
    Ok((receiver, router))
}

async fn send_message(sender: &GossipSender, message: Message) -> Result {
    sender.broadcast(message.to_vec().into()).await?;
    Ok(())
}
async fn receive_message(receiver: &mut GossipReceiver) -> Option<Message> {
    if let Some(event) = receiver.next().await {
        match event.unwrap() {
            // FIX: why the fuck is it unwrap
            Event::Received(message) => match Message::from_bytes(&message.content) {
                Ok(msg) => {
                    return Some(msg);
                }
                Err(e) => println!("failed to deserialize: {}", e),
            },
            _ => {}
        }
    }
    None
}

pub async fn create_receiver(
    joinargs: &JoinArgs,
) -> Result<(GossipSender, GossipReceiver, Router)> {
    let endpoint = Endpoint::builder(presets::N0).bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes([23u8; 32]);

    let (sender, receiver) = gossip
        .subscribe(topic_id, vec![get_secret(&joinargs.room).public()])
        .await?
        .split();
    // receiver.joined().await.unwrap();
    // let a = router
    //     .endpoint()
    //     .remote_info(get_secret(&joinargs.room).public())
    //     .await;
    // println!("{:?}", a);
    Ok((sender, receiver, router))
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    user_input.trim().to_string()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, override_usage = "<COMMAND>")]
struct Args {
    #[command(subcommand)]
    pub command: HostCommands,
}

#[derive(Subcommand, Debug)]
enum HostCommands {
    /// List users in the lobby (not implemented)
    List,
    /// Quit the Program
    Quit,
    /// Send a message (Not implemented)
    Message {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = clap::ArgAction::Append)]
        text: Vec<String>,
    },
    /// Start a pomodoro
    Start {
        #[arg(default_value = "50m")]
        study: String,
        #[arg(default_value = "10m")]
        rest: String,
    },
    /// Stop all pomotimers
    Stop,
}

pub async fn host_iroh(hostargs: HostArgs) -> Result {
    let (mut entryreceiver, entryrouter) = create_entry(&hostargs).await?;
    let (sender, mut receiver, router) = create_sender_receiver(&hostargs).await?;

    println!("Waiting to join.");
    entryreceiver.joined().await?;
    println!("Joined");

    loop {
        let msg = input(format!("{} ", "❯".with(crossterm::style::Color::AnsiValue(141))).as_str());
        if let Ok(args) =
            Args::try_parse_from(["Pomotimer"].into_iter().chain(msg.split_whitespace()))
                .inspect_err(|e| println!("{}", e))
        {
            match args.command {
                HostCommands::Quit => {
                    let _ = entryrouter.shutdown().await;
                    let _ = router.shutdown().await;
                    break;
                }
                HostCommands::Message { text } => {
                    if send_message(&sender, Message::new(MessageBody::Message { text }))
                        .await
                        .is_ok()
                    {
                        println!("Sent a message")
                    };
                }
                HostCommands::Start { study, rest } => {
                    let studydur: std::time::Duration = parse_duration(&study)
                        .inspect_err(|_| {
                            println!("Invalid time, using 50m instead.");
                        })
                        .unwrap_or(std::time::Duration::from_mins(50));
                    let restdur: std::time::Duration = parse_duration(&rest)
                        .inspect_err(|_| {
                            println!("Invalid time, using 10m instead.");
                        })
                        .unwrap_or(std::time::Duration::from_mins(50));
                    if send_message(
                        &sender,
                        Message::new(MessageBody::Pomodoro {
                            study: studydur,
                            rest: restdur,
                        }),
                    )
                    .await
                    .is_ok()
                    {};
                }
                HostCommands::Stop => {
                    send_message(&sender, Message::new(MessageBody::Stop))
                        .await
                        .ok();
                }
                _ => println!("Sorry I haven't implemented this yet. Coming soon tho!!"),
            }
        }
    }

    // receive_message(receiver, vec![entryrouter]).await?;

    Ok(())
}
async fn receive_till_stop(receiver: &mut GossipReceiver) -> Result {
    loop {
        if let Some(event) = receiver.next().await {
            match event? {
                Event::Received(message) => {
                    if let Ok(msg) = Message::from_bytes(&message.content) {
                        if let Message {
                            body: MessageBody::Stop,
                            ..
                        } = msg
                        {
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

async fn pomotimer(dur: Duration) {
    for i in 0..dur.as_secs() {
        println!("{i}/{}", dur.as_secs());
        sleep(Duration::from_secs(1)).await;
    }
}
pub async fn join_iroh(joinargs: JoinArgs) -> Result {
    let (_, mut receiver, router) = create_receiver(&joinargs).await?;

    println!("Waiting to join.");
    receiver.joined().await?;
    println!("Joined");

    loop {
        if let Some(Message { body, .. }) = receive_message(&mut receiver).await {
            match body {
                MessageBody::Pomodoro { study, rest } => {
                    let timer = Some(tokio::spawn(async move {
                        pomotimer(study.clone()).await;
                        pomotimer(rest.clone()).await;
                    }));
                    receive_till_stop(&mut receiver).await.ok(); // TODO: uhh it doesn't restart
                                                                 // unless completelly fucking
                                                                 // stoppped.
                    println!("host: stopped the pomodoro");
                    if let Some(t) = timer {
                        t.abort();
                    }
                }
                MessageBody::Message { text } => {
                    println!("host: {}", text.join(" "));
                }
                MessageBody::Stop => {}
            }
        }
    }
    // Ok(())
}

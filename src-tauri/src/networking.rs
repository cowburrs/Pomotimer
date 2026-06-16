// TODO: Implement drop for router so i can drop properly.
// TODO: I NEED TO REMOVE TAURI STATE FROM ALL FUNCTIONS AND WRAP TAURI STATE SO THAT I CAN CALL THE
// FUNCTIONS TO TEST INSTEAD OF JUST HOPING AND PRAYING

use std::time::Duration;

use futures::{FutureExt, StreamExt};
use iroh::{endpoint::presets, protocol::Router, Endpoint, PublicKey, SecretKey};
use iroh_gossip::{
    api::{Event::Received, GossipReceiver, GossipSender},
    Gossip, TopicId,
};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

struct Pc {
    router: Router,
    receiver: GossipReceiver,
    sender: GossipSender,
}
pub struct Pomoconnection {
    pc: Mutex<Option<Pc>>,
}
pub fn create_pc() -> Pomoconnection {
    Pomoconnection {
        pc: Mutex::new(None),
    }
}

fn secret_key_from_passphrase(passphrase: &str) -> SecretKey {
    let hash = Sha256::digest(passphrase.as_bytes());
    SecretKey::from_bytes(&hash.into())
}
fn get_secret(string: &str) -> SecretKey {
    secret_key_from_passphrase(&("pomotimer".to_owned() + string))
}

pub async fn network(
    state: tauri::State<'_, Pomoconnection>,
    bootstrap: Vec<PublicKey>,
    sk: Option<SecretKey>,
) -> Result<(), String> {
    let mut lock = match state.pc.try_lock() {
        Ok(thing) => thing,
        Err(_) => {
            eprintln!("Already trying to network!");
            return Err("tried to network twice.".to_string());
        }
    };
    println!("Lock obtained, proceeding to network.");
    if let Some(Pc { .. }) = &*lock {
        eprintln!("Can't host twice.");
        return Err("Hosted/joined twice".to_string());
    }
    let secret = match sk {
        None => SecretKey::generate(),
        Some(key) => key,
    };
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(secret)
        .bind()
        .await
        .map_err(|e| e.to_string())?;
    println!("Connected the endpoint.");
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes(sha2::Sha256::digest(b"pomotimer").into());
    let (sender, mut receiver) = gossip
        .subscribe(topic_id, bootstrap)
        .await
        .map_err(|e| e.to_string())?
        .split();
    println!("Waiting for someone to join/joining.");
    receiver.joined().await.map_err(|e| e.to_string())?;
    println!("Connected!");
    sender
        .broadcast(b"Awesomesauce".to_vec().into())
        .await
        .unwrap();
    match &*lock {
        None => {
            *lock = Some(Pc {
                router,
                receiver,
                sender,
            });
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
/// Host a connection, this will error if you try to run this/join twice.
pub async fn host(state: tauri::State<'_, Pomoconnection>, room: String) -> Result<(), String> {
    network(state, vec![], Some(get_secret(&room.to_string()))).await
}

#[tauri::command]
#[specta::specta]
/// Join a connection, you can string both the top and the bottom to join the host
pub async fn join(state: tauri::State<'_, Pomoconnection>, room: String) -> Result<(), String> {
    network(
        state,
        vec![get_secret(&room.to_string()).public()],
        Some(SecretKey::generate()),
    )
    .await
}

#[tauri::command]
#[specta::specta]
/// This is to receive the message, it will return the string or empty if there wasnt any messasges
pub fn receivemessage(state: tauri::State<'_, Pomoconnection>) -> Option<String> {
    let mut lock = match state.pc.try_lock() {
        Ok(lock) => lock,
        Err(_) => return None,
    };
    match &mut *lock {
        Some(Pc { receiver, .. }) => {
            if let Some(Some(Ok(Received(message)))) = receiver.next().now_or_never() {
                return Some(String::from_utf8(message.content.to_vec()).unwrap_or_default());
            };
        }
        None => return None,
    }

    None
}

#[tauri::command]
#[specta::specta]
/// This is to send a connection, both host and join can send one. plain text.
pub async fn sendmessage(
    state: tauri::State<'_, Pomoconnection>,
    message: &str,
) -> Result<(), String> {
    let mut lock = state.pc.lock().await;
    match &mut *lock {
        Some(Pc { sender, .. }) => sender
            .broadcast(message.as_bytes().to_vec().into())
            .await
            .map_err(|e| e.to_string())?,
        None => {
            return Err("No host/join endpoint to use!".to_string());
        }
    }

    Ok(())
}

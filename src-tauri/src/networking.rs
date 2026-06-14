// TODO: I NEED TO REMOVE TAURI STATE FROM ALL FUNCTIONS AND WRAP TAURI STATE SO THAT I CAN CALL THE
// FUNCTIONS TO TEST INSTEAD OF JUST HOPING AND PRAYING

use futures::{FutureExt, StreamExt};
use iroh::{endpoint::presets, protocol::Router, Endpoint, SecretKey};
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

#[tauri::command]
#[specta::specta]
/// Host a connection, this will error if you try to run this/join twice.
pub async fn host(state: tauri::State<'_, Pomoconnection>, room: String) -> Result<(), String> {
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(get_secret(&room))
        .bind()
        .await
        .map_err(|e| e.to_string())?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes(sha2::Sha256::digest(b"pomotimer").into());
    let (sender, mut receiver) = gossip
        .subscribe(topic_id, vec![])
        .await
        .map_err(|e| e.to_string())?
        .split();
    receiver.joined().await.map_err(|e| e.to_string())?;
    sender
        .broadcast(b"Awesomesauce".to_vec().into())
        .await
        .unwrap();
    let mut lock = state.pc.lock().await;
    match &*lock {
        Some(Pc { .. }) => return Err("Can't host join/twice idiot chara".to_string()),
        None => {
            *lock = Some(Pc {
                router,
                receiver,
                sender,
            });
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
/// Join a connection, you can string both the top and the bottom to join the host
pub async fn join(state: tauri::State<'_, Pomoconnection>, room: String) -> Result<(), String> {
    let endpoint = Endpoint::builder(presets::N0)
        .bind()
        .await
        .map_err(|e| e.to_string())?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();
    let topic_id = TopicId::from_bytes(sha2::Sha256::digest(b"pomotimer").into());
    let (sender, mut receiver) = gossip
        .subscribe(topic_id, vec![get_secret(&room).public()])
        .await
        .map_err(|e| e.to_string())?
        .split();
    receiver.joined().await.map_err(|e| e.to_string())?;
    sender
        .broadcast(b"Awesomesauce".to_vec().into())
        .await
        .unwrap();
    let mut lock = state.pc.lock().await;
    match &*lock {
        Some(Pc { .. }) => return Err("Can't join/host twice idiot chara".to_string()),
        None => {
            *lock = Some(Pc {
                router,
                receiver,
                sender,
            });
        }
    }

    Ok(())
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
    message: String,
) -> Result<(), String> {
    let mut lock = state.pc.lock().await;
    match &mut *lock {
        Some(Pc { sender, .. }) => sender
            .broadcast(message.into_bytes().into())
            .await
            .map_err(|e| e.to_string())?,
        None => {
            return Err("No host/join endpoint to use!".to_string());
        }
    }

    Ok(())
}

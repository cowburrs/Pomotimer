use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointAddr, EndpointId, SecretKey};
use iroh_gossip::{api::Event, Gossip, TopicId};
use n0_error::{Result, StdResultExt};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    body: MessageBody,
    nonce: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
enum MessageBody {
    AboutMe { name: String },
    Message { text: String },
}

impl Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string().into())
    }

    pub fn new(body: MessageBody) -> Self {
        Self {
            body,
            nonce: rand::random(),
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

fn bootstrap_peers() -> Vec<EndpointId> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        vec![args[1].parse().unwrap()]
    } else {
        vec![]
    }
}
pub async fn host_iroh() -> Result<()> {
    // create an iroh endpoint that includes the standard discovery mechanisms
    // we've built at number0
    let secret_key = secret_key_from_passphrase("pomotimer");
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(secret_key)
        .bind()
        .await?;
    println!("{}", endpoint.addr().id);

    // build gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // and you need some bootstrap peers to join the swarm
    let bootstrap_peers = vec![];

    // setup router
    let router = Router::builder(endpoint)
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();

    // gossip swarms are centered around a shared "topic id", which is a 32 byte identifier
    let topic_id = TopicId::from_bytes([23u8; 32]);

    // then, you can subscribe to the topic and join your initial peers
    let (sender, mut receiver) = gossip.subscribe(topic_id, bootstrap_peers).await?.split();

    // you might want to wait until you joined at least one other peer:
    receiver.joined().await?;

    // then, you can broadcast messages to all other peers!
    // let msg = b"hello world this is a gossip message";
    // println!("broadcasting: {:?}", std::str::from_utf8(msg));
    // sender.broadcast(msg.to_vec().into()).await?;
    let message = Message::new(MessageBody::AboutMe {
        name: String::from("alice"),
    });
    // Turn the message into a Vec, and then use
    // into to coerce the Vec into Bytes
    sender.broadcast(message.to_vec().into()).await?;

    // and read messages from others
    while let Some(event) = receiver.next().await {
        match event? {
            Event::Received(message) => {
                println!(
                    "received a message: {:?}",
                    std::str::from_utf8(&message.content)
                );
                let text = format!(
                    "{:?} {}",
                    std::time::Instant::now(),
                    String::from_utf8_lossy(&message.content)
                );
                sender.broadcast(text.into_bytes().into()).await?;
            }
            _ => {}
        }
    }

    // clean shutdown makes sure that other peers are notified that you went offline
    router.shutdown().await.std_context("shutdown router")?;
    Ok(())
}

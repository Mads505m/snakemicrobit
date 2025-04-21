use probe_rs::{Probe, Session, Permissions};
use probe_rs_rtt::Rtt;
use std::thread;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::{ws::Message, Filter};

#[tokio::main]
async fn main() {
    // Channel for broadcasting RTT lines
    let (tx, _) = broadcast::channel::<String>(16);

    // WebSocket route: ws://localhost:3030/ws
    let ws_route = {
        let tx = tx.clone();
        warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let mut rx = tx.subscribe();
                ws.on_upgrade(move |websocket| async move {
                    let (mut tx_ws, _) = websocket.split();
                    while let Ok(msg) = rx.recv().await {
                        if tx_ws.send(Message::text(msg)).await.is_err() {
                            break;
                        }
                    }
                })
            })
    };

    // 🔄 RTT reader thread
    thread::spawn(move || {
        // 1️⃣ Find and open the first connected debug probe
        let probe_info = Probe::list_all().get(0)
            .expect("❌ No probe found")
            .clone();
        let probe = probe_info.open().expect("❌ Failed to open probe");

        // 2️⃣ Attach to nRF52 target (micro:bit v2 = nrf52833_xxAA) with appropriate permissions
        let target = "nrf52833_xxAA";
        let permissions = Permissions::default(); // Set to READ_WRITE for memory read and write permissions
        let mut session = probe.attach(target, permissions).expect("❌ Failed to attach");

        // 3️⃣ Extract the memory map first
        let memory_map = session.target().memory_map.clone(); // Clone memory_map before mutable borrow of core

        // 4️⃣ Now borrow `core` mutably
        let mut core = session.core(0).expect("❌ Failed to get core");

        // 5️⃣ Attach RTT
        let mut rtt = loop {
            match Rtt::attach(&mut core, &memory_map) {
                Ok(rtt) => break rtt,
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        };

        // 6️⃣ Read from the first RTT up-channel
        let mut up = rtt.up_channels().take(0).unwrap();
        let mut buf = [0u8; 64];

        loop {
            match up.read(&mut core, &mut buf) {
                Ok(n) if n > 0 => {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    println!("📤 From RTT: {}", msg.trim());
                    let _ = tx.send(msg);
                }
                _ => thread::sleep(Duration::from_millis(10)),
            }
        }
    });

    println!("📡 WebSocket kører på ws://localhost:3030/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

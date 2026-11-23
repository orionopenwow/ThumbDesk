use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;

use crate::models::companion::{
    CompanionConnectionState, CompanionEdge, CompanionMessage, CompanionMessageType, CompanionMode,
    CompanionPeer, COMPANION_PROTOCOL_VERSION,
};

pub const DISCOVERY_UDP_PORT: u16 = 45912;
const BUILD_VERSION: &str = "1.0.0-win";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryBeacon {
    service: String,
    name: String,
    port: u16,
    protocol_version: u32,
}

struct CompanionInner {
    mode: CompanionMode,
    edge: CompanionEdge,
    state: CompanionConnectionState,
    peers: BTreeMap<String, (CompanionPeer, Instant)>,
    tx: Option<mpsc::UnboundedSender<CompanionMessage>>,
    generation: u64,
}

#[derive(Clone)]
pub struct CompanionService {
    inner: Arc<Mutex<CompanionInner>>,
    on_message: Arc<dyn Fn(CompanionMessage) + Send + Sync>,
    rt: Arc<tokio::runtime::Runtime>,
}

impl CompanionService {
    pub fn new<F>(on_message: F) -> Self
    where
        F: Fn(CompanionMessage) + Send + Sync + 'static,
    {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create companion Tokio runtime");

        Self {
            inner: Arc::new(Mutex::new(CompanionInner {
                mode: CompanionMode::Off,
                edge: CompanionEdge::Right,
                state: CompanionConnectionState::Off,
                peers: BTreeMap::new(),
                tx: None,
                generation: 0,
            })),
            on_message: Arc::new(on_message),
            rt: Arc::new(rt),
        }
    }

    pub fn mode(&self) -> CompanionMode {
        self.inner.lock().unwrap().mode
    }

    pub fn edge(&self) -> CompanionEdge {
        self.inner.lock().unwrap().edge
    }

    pub fn set_edge(&self, edge: CompanionEdge) {
        self.inner.lock().unwrap().edge = edge;
    }

    pub fn state(&self) -> CompanionConnectionState {
        self.inner.lock().unwrap().state.clone()
    }

    pub fn peers(&self) -> Vec<CompanionPeer> {
        let mut guard = self.inner.lock().unwrap();
        let now = Instant::now();
        guard
            .peers
            .retain(|_, (_, seen)| now.duration_since(*seen) < Duration::from_secs(10));
        let mut list: Vec<CompanionPeer> = guard.peers.values().map(|(p, _)| p.clone()).collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list
    }

    pub fn send(&self, msg: CompanionMessage) -> bool {
        let tx = self.inner.lock().unwrap().tx.clone();
        if let Some(sender) = tx {
            sender.send(msg).is_ok()
        } else {
            false
        }
    }

    pub fn disconnect(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.tx = None;
        guard.generation = guard.generation.wrapping_add(1);
        guard.state = match guard.mode {
            CompanionMode::Off => CompanionConnectionState::Off,
            CompanionMode::Controller => CompanionConnectionState::Browsing,
            CompanionMode::Receiver => CompanionConnectionState::Listening,
        };
    }

    pub fn set_mode(&self, mode: CompanionMode) {
        let gen = {
            let mut guard = self.inner.lock().unwrap();
            guard.mode = mode;
            guard.tx = None;
            guard.peers.clear();
            guard.generation = guard.generation.wrapping_add(1);
            guard.state = match mode {
                CompanionMode::Off => CompanionConnectionState::Off,
                CompanionMode::Controller => CompanionConnectionState::Browsing,
                CompanionMode::Receiver => CompanionConnectionState::Listening,
            };
            guard.generation
        };

        match mode {
            CompanionMode::Off => {}
            CompanionMode::Receiver => self.start_receiver(gen),
            CompanionMode::Controller => self.start_browser(gen),
        }
    }

    pub fn connect_to_peer(&self, peer_id: Option<&str>) {
        let target = {
            let peers = self.peers();
            if let Some(id) = peer_id {
                peers.into_iter().find(|p| p.id == id)
            } else {
                peers.into_iter().next()
            }
        };
        let Some(peer) = target else {
            return;
        };
        let gen = {
            let mut guard = self.inner.lock().unwrap();
            guard.state = CompanionConnectionState::Connecting(peer.name.clone());
            guard.generation
        };
        let svc = self.clone();
        self.rt.spawn(async move {
            let addr = format!("{}:{}", peer.address, peer.port);
            match TcpStream::connect(&addr).await {
                Ok(stream) => {
                    svc.attach_stream(stream, peer.name.clone(), gen).await;
                }
                Err(e) => {
                    let mut guard = svc.inner.lock().unwrap();
                    if guard.generation == gen {
                        guard.state = CompanionConnectionState::Error(format!("Connect failed: {}", e));
                    }
                }
            }
        });
    }

    fn local_host_name() -> String {
        std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows PC".to_string())
    }

    fn start_receiver(&self, gen: u64) {
        let svc = self.clone();
        self.rt.spawn(async move {
            let listener = match TcpListener::bind("0.0.0.0:0").await {
                Ok(l) => l,
                Err(e) => {
                    let mut guard = svc.inner.lock().unwrap();
                    if guard.generation == gen {
                        guard.state = CompanionConnectionState::Error(format!("Bind failed: {}", e));
                    }
                    return;
                }
            };
            let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);

            // Spawn UDP LAN discovery beacon
            let beacon_inner = Arc::clone(&svc.inner);
            tokio::spawn(async move {
                if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                    let _ = sock.set_broadcast(true);
                    let beacon = DiscoveryBeacon {
                        service: "_vibectl._tcp".to_string(),
                        name: Self::local_host_name(),
                        port,
                        protocol_version: COMPANION_PROTOCOL_VERSION,
                    };
                    let payload = serde_json::to_vec(&beacon).unwrap_or_default();
                    let dest = format!("255.255.255.255:{}", DISCOVERY_UDP_PORT);
                    loop {
                        if beacon_inner.lock().unwrap().generation != gen {
                            break;
                        }
                        let _ = sock.send_to(&payload, &dest).await;
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            });

            loop {
                if svc.inner.lock().unwrap().generation != gen {
                    break;
                }
                if let Ok(Ok((stream, addr))) =
                    tokio::time::timeout(Duration::from_millis(250), listener.accept()).await
                {
                    let peer_name = addr.ip().to_string();
                    let svc_conn = svc.clone();
                    tokio::spawn(async move {
                        svc_conn.attach_stream(stream, peer_name, gen).await;
                    });
                }
            }
        });
    }

    fn start_browser(&self, gen: u64) {
        let svc = self.clone();
        self.rt.spawn(async move {
            let sock = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_UDP_PORT)).await {
                Ok(s) => s,
                Err(_) => return,
            };
            let mut buf = [0u8; 1024];
            loop {
                if svc.inner.lock().unwrap().generation != gen {
                    break;
                }
                match tokio::time::timeout(Duration::from_millis(500), sock.recv_from(&mut buf))
                    .await
                {
                    Ok(Ok((n, addr))) => {
                        if let Ok(beacon) = serde_json::from_slice::<DiscoveryBeacon>(&buf[..n]) {
                            if beacon.service == "_vibectl._tcp" {
                                let peer = CompanionPeer {
                                    id: format!("{}:{}", addr.ip(), beacon.port),
                                    name: beacon.name,
                                    address: addr.ip().to_string(),
                                    port: beacon.port,
                                };
                                let should_autoconnect = {
                                    let mut guard = svc.inner.lock().unwrap();
                                    guard.peers.insert(peer.id.clone(), (peer.clone(), Instant::now()));
                                    guard.tx.is_none()
                                        && matches!(guard.state, CompanionConnectionState::Browsing)
                                };
                                if should_autoconnect {
                                    svc.connect_to_peer(Some(&peer.id));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    async fn attach_stream(&self, stream: TcpStream, initial_name: String, gen: u64) {
        let (reader, mut writer) = stream.into_split();
        let (tx, mut rx) = mpsc::unbounded_channel::<CompanionMessage>();

        {
            let mut guard = self.inner.lock().unwrap();
            if guard.generation != gen {
                return;
            }
            guard.tx = Some(tx.clone());
            guard.state = CompanionConnectionState::Connected(initial_name);
        }

        // Send Hello immediately on ready
        let _ = tx.send(CompanionMessage::hello(
            Self::local_host_name(),
            BUILD_VERSION.to_string(),
        ));

        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(mut line) = serde_json::to_string(&msg) {
                    line.push('\n');
                    if writer.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        });

        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if self.inner.lock().unwrap().generation != gen {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<CompanionMessage>(&line) {
                Ok(msg) => {
                    if msg.message_type == CompanionMessageType::Hello {
                        if let Some(name) = &msg.name {
                            let mut guard = self.inner.lock().unwrap();
                            if guard.generation == gen {
                                guard.state = CompanionConnectionState::Connected(name.clone());
                            }
                        }
                    }
                    (self.on_message)(msg);
                }
                Err(_) => {
                    let mut guard = self.inner.lock().unwrap();
                    if guard.generation == gen {
                        guard.state =
                            CompanionConnectionState::Error("Invalid companion packet".to_string());
                    }
                    break;
                }
            }
        }

        writer_task.abort();
        let mut guard = self.inner.lock().unwrap();
        if guard.generation == gen {
            guard.tx = None;
            guard.state = match guard.mode {
                CompanionMode::Off => CompanionConnectionState::Off,
                CompanionMode::Controller => CompanionConnectionState::Browsing,
                CompanionMode::Receiver => CompanionConnectionState::Listening,
            };
        }
    }

    /// Direct loopback connect helper for integration tests.
    pub fn connect_direct_addr(&self, addr: SocketAddr, name: String) {
        let gen = self.inner.lock().unwrap().generation;
        let svc = self.clone();
        self.rt.spawn(async move {
            if let Ok(stream) = TcpStream::connect(addr).await {
                svc.attach_stream(stream, name, gen).await;
            }
        });
    }
}

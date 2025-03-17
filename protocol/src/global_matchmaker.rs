use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use iroh::{
    endpoint::VarInt,
    Endpoint, NodeId, PublicKey, SecretKey,
};
use n0_future::{task::AbortOnDropHandle, FuturesUnordered, StreamExt};
use rand::Rng;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    _bootstrap_keys::BOOTSTRAP_SECRET_KEYS,
    _random_word::get_nickname_from_pubkey,
    chat::{ChatEventStream, ChatSender, ChatTicket},
    echo::Echo,
    main_node::MainNode,
};

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct GlobalMatchmaker(Arc<Mutex<GlobalMatchmakerInner>>);

struct GlobalMatchmakerInner {
    own_private_key: SecretKey,
    own_endpoint: Option<MainNode>,
    bootstrap_key: Option<SecretKey>,
    bootstrap_endpoint: Option<MainNode>,
    known_bootstrap_nodes: BTreeMap<usize, BootstrapNodeInfo>,
    _periodic_task: Option<AbortOnDropHandle<()>>,
    global_chat_controller: Option<GlobalChatController>,
    _bs_global_chat_task: Option<AbortOnDropHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct BootstrapNodeInfo {
    bs_idx: usize,
    bootstrap_id: NodeId,
    own_id: NodeId,
    ping_secs: f32,
    connect_secs: f32,
}

const GLOBAL_PERIODIC_TASK_INTERVAL: Duration = Duration::from_secs(5);

async fn global_periodic_task(_mm: GlobalMatchmaker) {
    loop {
        let interval =
            GLOBAL_PERIODIC_TASK_INTERVAL + Duration::from_secs(rand::thread_rng().gen_range(0..5));
        n0_future::time::sleep(interval).await;
        match global_periodic_task_iteration_1(_mm.clone()).await {
            Ok(_) => {}
            Err(e) => {
                warn!("global periodic task iteration 1 failed: {e}");
            }
        }
        let interval =
            GLOBAL_PERIODIC_TASK_INTERVAL + Duration::from_secs(rand::thread_rng().gen_range(0..5));
        n0_future::time::sleep(interval).await;
        match global_periodic_task_iteration_2(_mm.clone()).await {
            Ok(_) => {}
            Err(e) => {
                warn!("global periodic task iteration 2 failed: {e}");
            }
        }
    }
}

async fn global_periodic_task_iteration_1(mm: GlobalMatchmaker) -> Result<()> {
    mm.connect_to_bootstrap().await?;
    Ok(())
}

async fn global_periodic_task_iteration_2(mm: GlobalMatchmaker) -> Result<()> {
    if mm.bs_endpoint().await.is_none() {
        mm.connect_to_bootstrap().await?;
        let added = mm.spawn_bootstrap_endpoint().await?;
        if added {
            mm.connect_bootstrap_chat().await?;
        }
    }

    Ok(())
}

impl GlobalMatchmaker {
    pub async fn display_debug_info(&self) -> String {
        let endpoint = self.own_endpoint().await.node_id();
        let bs_endpoint = self.bs_endpoint().await.map(|bs| bs.node_id());
        let bs = self.known_bootstrap_nodes().await;
        let mut info_txt = String::new();
        info_txt.push_str(&format!("Own Endpoint NodeID: \n{endpoint:#?}\n\n"));
        info_txt.push_str(&format!("Own Bootstrap NodeID: \n{bs_endpoint:#?}\n\n"));
        info_txt.push_str(&format!("Known Bootstrap Nodes: \n{bs:#?}\n\n"));
        info_txt
    }
    async fn fresh(own_private_key: SecretKey, nickname: String) -> Result<Self> {
        let mm = Self(Arc::new(Mutex::new(GlobalMatchmakerInner {
            own_private_key: own_private_key.clone(),
            own_endpoint: None,
            bootstrap_key: None,
            bootstrap_endpoint: None,
            known_bootstrap_nodes: BTreeMap::new(),
            _periodic_task: None,
            global_chat_controller: None,
            _bs_global_chat_task: None,
        })));

        let own_endpoint = MainNode::spawn(nickname, own_private_key.clone(), None).await?;
        {
            mm.0.lock().await.own_endpoint = Some(own_endpoint)
        };
        Ok(mm)
    }
    pub async fn bootstrap_nodes_set(&self) -> BTreeSet<NodeId> {
        self.0
            .lock()
            .await
            .known_bootstrap_nodes
            .values()
            .map(|bs| vec![bs.bootstrap_id, bs.own_id])
            .collect::<Vec<_>>()
            .iter()
            .flatten()
            .copied()
            .collect()
    }
    pub async fn own_endpoint(&self) -> Endpoint {
        self.0
            .lock()
            .await
            .own_endpoint
            .as_ref()
            .unwrap()
            .endpoint()
            .clone()
    }
    pub async fn own_node(&self) -> MainNode {
        self.0.lock().await.own_endpoint.as_ref().unwrap().clone()
    }
    pub async fn bs_node(&self) -> Option<MainNode> {
        self.0
            .lock()
            .await
            .bootstrap_endpoint
            .as_ref()
            .map(|bs| bs.clone())
    }
    pub async fn bs_endpoint(&self) -> Option<Endpoint> {
        self.0
            .lock()
            .await
            .bootstrap_endpoint
            .as_ref()
            .map(|bs| bs.endpoint().clone())
    }
    pub async fn own_private_key(&self) -> SecretKey {
        self.0.lock().await.own_private_key.clone()
    }

    pub async fn new(own_private_key: SecretKey) -> Result<Self> {
        let num = 3;
        for i in 0..num {
            match Self::new_try_once(
                own_private_key.clone(),
                get_nickname_from_pubkey(own_private_key.public()),
            )
            .await
            {
                Ok(mm) => {
                    return Ok(mm);
                }
                Err(e) => {
                    warn!("failed to create global matchmaker, retrying {i}/{num}... {e}");
                    n0_future::time::sleep(Duration::from_secs(1 + i)).await;
                }
            }
        }
        anyhow::bail!("failed to create global matchmaker after 3 attempts");
    }
    async fn new_try_once(own_private_key: SecretKey, nickname: String) -> Result<Self> {
        info!(
            "Creating new global matchmaker, we are {}",
            own_private_key.public()
        );
        let mm = Self::fresh(own_private_key, nickname.clone()).await?;
        let mm = if let Ok(_) = mm.connect_to_bootstrap().await {
            info!("Successfully connected to foreign bootstrap node");
            mm
        } else {
            mm.spawn_bootstrap_endpoint().await?;

            mm
        };

        mm.connect_global_chats().await?;

        let periodic_task =
            AbortOnDropHandle::new(n0_future::task::spawn(global_periodic_task(mm.clone())));
        {
            mm.0.lock().await._periodic_task = Some(periodic_task);
        }

        Ok(mm)
    }

    async fn connect_global_chats(&self) -> Result<()> {
        self.connect_bootstrap_chat().await?;
        info!("connect_global_chats(): joining normal chat");
        let ticket = self.get_global_chat_ticket().await?;
        let (sender, receiver) = self.own_node().await.join_chat(&ticket)?;
        let c1 = GlobalChatController { sender, receiver };
        {
            info!("connect_global_chats(): saving normal chat controller...");
            self.0.lock().await.global_chat_controller = Some(c1);
            info!("connect_global_chats(): saved.");
        }

        info!("connect_global_chats(): done.");
        Ok(())
    }

    pub async fn get_global_chat_ticket(&self) -> Result<ChatTicket> {
        let nodes = self.bootstrap_nodes_set().await;
        let ticket = ChatTicket::new_str_bs("global-chat", nodes);
        Ok(ticket)
    }

    async fn connect_bootstrap_chat(&self) -> Result<()> {
        if let Some(node2) = self.bs_node().await {
            info!("connect_global_chats(): joining bootstrap chat");
            let ticket = self.get_global_chat_ticket().await?;
            let (sender, mut receiver) = node2.join_chat(&ticket)?;
            let _task = AbortOnDropHandle::new(n0_future::task::spawn(async move {
                match sender.send("Hello, world!".to_string()).await {
                    Ok(_) => {
                        info!("BOOTSTRAP: sent hello world OK");
                    }
                    Err(e) => {
                        warn!("BOOTSTRAP: failed to send hello world: {e}");
                    }
                }
                let mut i = 0;
                while let Some(event) = receiver.next().await {
                    i += 1;
                    if i % 666 == 0 {
                        info!("BOOTSTRAP: global chat event: {event:?}");
                        let _ = sender.send("Still here.".to_string()).await;
                    }
                }
            }));
            {
                self.0.lock().await._bs_global_chat_task = Some(_task);
            }
        } else {
            info!("connect_bootstrap_chat(): no bootstrap node, skipping bootstrap chat.");
        }
        Ok(())
    }

    pub async fn take_global_chat_controllers(&self) -> Option<GlobalChatController> {
        let mut v0 = None;
        std::mem::swap(&mut self.0.lock().await.global_chat_controller, &mut v0);
        v0
    }

    pub async fn known_bootstrap_nodes(&self) -> BTreeMap<usize, BootstrapNodeInfo> {
        self.0.lock().await.known_bootstrap_nodes.clone()
    }

    pub async fn spawn_bootstrap_endpoint(&self) -> Result<bool> {
        let own_id = self.own_endpoint().await.node_id();
        let nickname = self.own_node().await.nickname().to_string();
        let boostrap_idx = {
            let inner = self.0.lock().await;
            let all_bs_idx = BOOTSTRAP_SECRET_KEYS
                .iter()
                .enumerate()
                .map(|(i, _)| i)
                .collect::<HashSet<_>>();
            let present_bs_idx = inner
                .known_bootstrap_nodes
                .keys()
                .cloned()
                .collect::<HashSet<_>>();
            let free_bs_idx = all_bs_idx.difference(&present_bs_idx).collect::<Vec<_>>();
            if free_bs_idx.is_empty() {
                // info!("no free bootstrap idx, exiting.");
                return Ok(false);
            }
            let rand = rand::thread_rng().gen_range(0..free_bs_idx.len());
            *free_bs_idx[rand]
        };
        info!("Spawning new bootstrap endpoint #{boostrap_idx}");
        {
            let mut inner = self.0.lock().await;
            let bootstrap_key = SecretKey::from_bytes(&BOOTSTRAP_SECRET_KEYS[boostrap_idx]);

            let nickname = format!("{} (bootstrap)", nickname);
            let bootstrap_endpoint =
                MainNode::spawn(nickname, bootstrap_key.clone(), Some(own_id)).await?;
            inner.bootstrap_key = Some(bootstrap_key);
            inner.bootstrap_endpoint = Some(bootstrap_endpoint);
        }

        info!("Connecting to own bootstrap endpoint");
        self.connect_to_bootstrap().await?;
        info!("Successfully connected to own bootstrap endpoint");
        let known_bs = self.known_bootstrap_nodes().await;
        let our_bs = known_bs
            .get(&boostrap_idx)
            .context("faild to find ourselves")?;
        if our_bs.own_id != self.own_endpoint().await.node_id() {
            warn!("our own bootstrap node id does not match the known bootstrap node id");
            warn!(
                "\n our_bs.own_id: {:#?}\n own_endpoint: {:#?}",
                our_bs.own_id,
                self.own_endpoint().await.node_id()
            );
            let mut inner = self.0.lock().await;
            let old_endpoint = inner.bootstrap_endpoint.take();
            inner.bootstrap_endpoint = None;
            inner.bootstrap_key = None;
            if let Some(old_endpoint) = old_endpoint {
                old_endpoint.shutdown().await?;
            }
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn connect_to_bootstrap(&self) -> Result<()> {
        let mut fut = FuturesUnordered::new();
        let endpoint = self.own_endpoint().await;
        for (i, bs_known_secret) in BOOTSTRAP_SECRET_KEYS.iter().enumerate() {
            let bs_node_id = SecretKey::from_bytes(bs_known_secret).public();
            let endpoint = endpoint.clone();
            fut.push(async move {
                (
                    i,
                    (move || async move {
                        let t0 = n0_future::time::Instant::now();
                        let connection = n0_future::time::timeout(
                            CONNECT_TIMEOUT,
                            endpoint.connect(bs_node_id, Echo::ALPN),
                        )
                        .await??;
                        let t1 = n0_future::time::Instant::now();
                        let connect_secs = (t1 - t0).as_secs_f32();
                        let (mut send, mut recv) = connection.open_bi().await?;
                        let send_buf = endpoint.node_id().as_bytes().to_vec();
                        send.write_all(&send_buf).await?;
                        let mut recv_buf = [0; 32];
                        recv.read_exact(&mut recv_buf).await?;
                        let recv_pubkey = PublicKey::from_bytes(&recv_buf)?;
                        let t2 = n0_future::time::Instant::now();
                        let ping_secs = (t2 - t1).as_secs_f32();

                        connection.close(VarInt::from(0_u32), "ok".as_bytes());
                        drop(connection);

                        anyhow::Ok(BootstrapNodeInfo {
                            bootstrap_id: bs_node_id,
                            own_id: recv_pubkey,
                            bs_idx: i,
                            ping_secs,
                            connect_secs,
                        })
                    })()
                    .await,
                )
            });
        }
        while let Some((i, res)) = fut.next().await {
            match res {
                Ok(info) => {
                    let mut inner = self.0.lock().await;
                    let _r = inner.known_bootstrap_nodes.insert(info.bs_idx, info);
                    if _r.is_none() {
                        info!("added connection to bootstrap node #{i}");
                    }
                }
                Err(_e) => {
                    let mut inner = self.0.lock().await;
                    let _r = inner.known_bootstrap_nodes.remove(&i);
                    if _r.is_some() {
                        warn!("removed bootstrap node #{i} from known bootstrap nodes: {_e}");
                    }
                    continue;
                }
            }
        }
        {
            let inner = self.0.lock().await;
            if inner.known_bootstrap_nodes.is_empty() {
                anyhow::bail!("failed to connect to any bootstrap node");
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("shutting down");
        {
            let mut inner = self.0.lock().await;

            if let Some(own_endpoint) = inner.own_endpoint.take() {
                own_endpoint.shutdown().await?;
            }
            if let Some(bootstrap_endpoint) = inner.bootstrap_endpoint.take() {
                bootstrap_endpoint.shutdown().await?;
            }
            if let Some(x) = inner.global_chat_controller.take() {
                drop(x);
            }
            if let Some(x) = inner._bs_global_chat_task.take() {
                drop(x);
            }
            if let Some(x) = inner._periodic_task.take() {
                drop(x);
            }
        }
        info!("shutdown complete");
        Ok(())
    }
}

pub struct GlobalChatController {
    pub sender: ChatSender,
    pub receiver: ChatEventStream,
}

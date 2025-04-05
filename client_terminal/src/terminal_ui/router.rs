use std::{collections::BTreeMap, sync::Arc};

use super::pages::{ChatPageFactory, SingleplayerPageFactory};
use async_trait::async_trait;
use crossterm::event::Event;
use n0_future::task::AbortOnDropHandle;
use tokio::sync::{futures::Notified, Mutex, Notify};

pub fn make_router() -> Router {
    let mut map: BTreeMap<PageRoute, Arc<dyn PageFactory>> = BTreeMap::new();
    map.insert(PageRoute::Chat, Arc::new(ChatPageFactory));
    map.insert(PageRoute::Singleplayer, Arc::new(SingleplayerPageFactory));
    Router::new(map)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageRoute {
    Chat,
    Singleplayer,
}

#[derive(Clone)]
pub struct Router {
    inner: Arc<Mutex<RouterInner>>,
    notify: Arc<Notify>,
}

pub struct RouterInner {
    pages: BTreeMap<PageRoute, Arc<dyn PageFactory>>,
    current_route: Option<PageRoute>,
    current_page: Option<DynamicPage>,
}

impl Router {
    pub async fn shutdown(&self) {
        let mut inner = self.inner.lock().await;
        if let Some(mut page) = inner.current_page.take() {
            page.shutdown().await;
        }
    }

    fn new(map: BTreeMap<PageRoute, Arc<dyn PageFactory>>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RouterInner {
                pages: map,
                current_route: None,
                current_page: None,
            })),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn open_page(&self, route: PageRoute) -> anyhow::Result<()> {
        {
            let mut inner = self.inner.lock().await;
            let inner = &mut *inner;
            if inner.current_route == Some(route) {
                return Ok(());
            }

            if let Some(mut prev_page) = inner.current_page.take() {
                n0_future::task::spawn(async move {
                    prev_page.shutdown().await;
                });
            }

            let page_factory = inner
                .pages
                .get(&route)
                .ok_or(anyhow::anyhow!("Page not found"))?;
            inner.current_route = Some(route);
            inner.current_page =
                Some(page_factory.create_page(self.notify.clone()));
        }
        self.notify.notify_waiters();
        Ok(())
    }

    pub fn handle_event_stream(
        &self,
        mut event_rx: tokio::sync::mpsc::Receiver<Event>,
    ) -> AbortOnDropHandle<()> {
        let r = self.clone();
        let notify = self.notify.clone();
        AbortOnDropHandle::new(n0_future::task::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                {
                    let mut inner = r.inner.lock().await;
                    let inner = &mut *inner;
                    if let Some(page) = inner.current_page.as_mut() {
                        page.handle_event(event).await;
                        notify.notify_waiters();
                    }
                }
            }
        }))
    }

    pub async fn get_page(&self) -> Option<DynamicPage> {
        let inner = self.inner.lock().await;
        inner.current_page.clone()
    }

    pub fn notified(&self) -> Notified {
        self.notify.notified()
    }

    pub async fn handle_navigation_event(
        &self,
        event: &Event,
    ) -> anyhow::Result<bool> {
        let Event::Key(key) = event else {
            return Ok(false);
        };
        let route = {
            if key.kind != crossterm::event::KeyEventKind::Press {
                return Ok(false);
            }
            match key.code {
                crossterm::event::KeyCode::F(1) => PageRoute::Singleplayer,
                crossterm::event::KeyCode::F(2) => PageRoute::Chat,
                _ => {
                    return Ok(false);
                }
            }
        };
        self.open_page(route).await?;
        Ok(true)
    }
}

#[async_trait]
pub trait PageFactory: std::fmt::Debug + Send + Sync + 'static {
    fn create_page(&self, notify: Arc<Notify>) -> DynamicPage;
}

#[derive(Clone)]
pub struct DynamicPage(Arc<dyn Page>, Arc<AbortOnDropHandle<()>>);

impl DynamicPage {
    pub fn new(page: impl Page, task: AbortOnDropHandle<()>) -> Self {
        Self(Arc::new(page), Arc::new(task))
    }
}

#[async_trait]
pub trait Page: Send + Sync + 'static {
    async fn get_drawable(&self) -> Box<dyn Drawable>;
    async fn shutdown(&self);
    async fn handle_event(&self, event: Event);
}

pub trait Drawable: std::fmt::Debug + Send + Sync + 'static {
    fn draw(&self, frame: &mut ratatui::Frame<'_>);
}

#[async_trait]
impl Page for DynamicPage {
    async fn get_drawable(&self) -> Box<dyn Drawable> {
        self.0.get_drawable().await
    }
    async fn shutdown(&self) {
        self.0.shutdown().await
    }
    async fn handle_event(&self, event: Event) {
        self.0.handle_event(event).await
    }
}

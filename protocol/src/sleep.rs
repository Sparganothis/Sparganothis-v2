use std::{sync::Arc, time::Duration};

use tokio::{sync::Notify};
use n0_future::time::Instant;

#[derive(Clone, Debug)]
pub struct SleepManager {
    inner: Arc<SleepManagerInner>,
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SleepManagerInner::new()),
        }
    }
    pub async fn sleep(&self, duration: Duration) {
        self.inner.sleep(duration).await;
    }
    pub fn wake_up(&self) {
        self.inner.wake_up();
    }
}

#[derive(Debug)]
struct SleepManagerInner {
    trigger: Notify,
}

impl SleepManagerInner {
    fn new() -> Self {
        Self {
            trigger: Notify::new(),
        }
    }
    async fn sleep(&self, duration: Duration) {
        let mut millis_left = duration.as_micros() as i128;
        while millis_left > 0 {
            let now = Instant::now();
            n0_future::future::race( n0_future::time::sleep(Duration::from_micros(millis_left as u64)), 
                self.trigger.notified(),
            ).await;
            millis_left -= now.elapsed().as_micros() as i128;
        }
    }
    fn wake_up(&self) {
        self.trigger.notify_waiters();
        self.trigger.notify_one();
    }
           
}

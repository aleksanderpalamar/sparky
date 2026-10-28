use super::proxy;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use zbus::Connection;

#[derive(Default)]
pub struct Discoveries {
    active: tokio::sync::Mutex<HashMap<String, u64>>,
    next: AtomicU64,
}

impl Discoveries {
    pub async fn start(
        self: &Arc<Self>,
        connection: Connection,
        adapter: String,
    ) -> zbus::Result<()> {
        let mut active = self.active.lock().await;
        if active.contains_key(&adapter) {
            return Ok(());
        }
        proxy(&connection, &adapter, "org.bluez.Adapter1")
            .await?
            .call::<_, _, ()>("StartDiscovery", &())
            .await?;
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        active.insert(adapter.clone(), id);
        drop(active);
        let discoveries = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(15)).await;
            let _ = discoveries.stop(&connection, &adapter, Some(id)).await;
        });
        Ok(())
    }

    pub async fn stop(
        &self,
        connection: &Connection,
        adapter: &str,
        id: Option<u64>,
    ) -> zbus::Result<()> {
        let mut active = self.active.lock().await;
        let Some(current) = active.get(adapter).copied() else {
            return Ok(());
        };
        if id.is_some_and(|id| id != current) {
            return Ok(());
        }
        let result = proxy(connection, adapter, "org.bluez.Adapter1")
            .await?
            .call::<_, _, ()>("StopDiscovery", &())
            .await;
        active.remove(adapter);
        result
    }

    pub async fn stop_all(&self, connection: &Connection) {
        let adapters: Vec<_> = self.active.lock().await.keys().cloned().collect();
        for adapter in adapters {
            let _ = self.stop(connection, &adapter, None).await;
        }
    }
}

//! Drain BACnetRouter local-delivery channel so dispatch cannot wedge.
//!
//! Upstream `BACnetRouter::start` returns a bounded (256) `mpsc` receiver. Local
//! final-hop and global-broadcast APDUs are sent with `.await`; if nothing
//! consumes them, forwarding stalls. A half-router appliance has no local
//! application database — count and discard.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bacnet_network::layer::ReceivedApdu;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Running drain of `local_rx` with a bounded counter (no unbounded buffering).
pub struct LocalDeliveryDrain {
    count: Arc<AtomicU64>,
    join: JoinHandle<()>,
}

impl LocalDeliveryDrain {
    /// Take ownership of `local_rx` and spawn a continuous consumer.
    #[must_use]
    pub fn spawn(mut local_rx: mpsc::Receiver<ReceivedApdu>) -> Self {
        let count = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&count);
        let join = tokio::spawn(async move {
            while local_rx.recv().await.is_some() {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        Self { count, join }
    }

    #[must_use]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Abort the drain task and wait for it to finish (best-effort).
    pub async fn stop(self) {
        self.join.abort();
        let _ = self.join.await;
    }
}

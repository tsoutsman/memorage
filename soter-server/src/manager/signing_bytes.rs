use std::time::Instant;

use soter_cs::SigningBytes;
use tokio::sync::{mpsc, oneshot};

/// How often the signing bytes are refreshed, expressed in seconds. The refresh will only trigger
/// if the signing bytes are requested.
const SIGNING_BYTES_REFRESH_TIME: u64 = 60;

pub async fn manager(mut rx: mpsc::Receiver<oneshot::Sender<SigningBytes>>) {
    let mut signing_bytes = SigningBytes::new();
    let mut time_generated = Instant::now();

    while let Some(resp) = rx.recv().await {
        if time_generated.elapsed().as_secs() >= SIGNING_BYTES_REFRESH_TIME {
            signing_bytes = SigningBytes::new();
            time_generated = Instant::now();
        }

        let _ = resp.send(signing_bytes);
    }
}

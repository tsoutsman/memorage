use std::time::Instant;

use soter_cs::SigningBytes;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, info_span};

/// How often the signing bytes are refreshed, expressed in seconds. The refresh will only trigger
/// if the signing bytes are requested.
const SIGNING_BYTES_REFRESH_TIME: u64 = 60;

pub async fn manager(mut rx: mpsc::Receiver<oneshot::Sender<SigningBytes>>) {
    let mut signing_bytes = SigningBytes::new();
    let mut time_generated = Instant::now();

    while let Some(resp) = rx.recv().await {
        let span = info_span!("received command").entered();

        if time_generated.elapsed().as_secs() >= SIGNING_BYTES_REFRESH_TIME {
            signing_bytes = SigningBytes::new();
            info!(
                life_of_previous_signing_bytes = %time_generated.elapsed().as_secs(),
                "regenerated signing bytes"
            );
            time_generated = Instant::now();
        }

        debug!(?signing_bytes, "sending signing bytes");

        let _ = resp.send(signing_bytes);
        drop(span);
    }
}

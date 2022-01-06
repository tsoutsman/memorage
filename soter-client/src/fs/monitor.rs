use std::path::Path;

use notify::{RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::error;

pub use notify::Event;

/// Returns a [`Receiver`](mpsc::Receiver) that will send any [`Events`](Event) that happen at the
/// given path.
pub fn changed_files<P>(path: P) -> crate::Result<mpsc::Receiver<Event>>
where
    P: AsRef<Path>,
{
    let (tx, rx) = mpsc::channel(16);
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            // TODO: Log?
            let _ = tx.blocking_send(event);
        }
        Err(error) => error!(?error, "error monitoring directory"),
    })?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    Ok(rx)
}

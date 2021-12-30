use std::time::Duration;

pub struct Config {
    pub server_ping_delay: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_ping_delay: Duration::from_secs(10),
        }
    }
}

use std::sync::Arc;

use iced::Subscription;
use iced_native::subscription;
use memorage_client::{
    net::{
        peer::{sleep_till, IncomingConnection},
        Client,
    },
    persistent::{config::Config, data::Data},
};
use memorage_core::{time::OffsetDateTime, Mutex};

#[derive(Debug)]
pub enum Event {
    Checked,
    Scheduled(OffsetDateTime),
    Connected,
    Complete,
    Error(memorage_client::Error),
}

enum State {
    Disconnected,
    Scheduled(Client<Data>, OffsetDateTime),
    Connected(IncomingConnection),
}

pub fn handle(data: Arc<Mutex<Data>>, config: Arc<Mutex<Config>>) -> Subscription<Event> {
    struct Connect;

    subscription::unfold(
        std::any::TypeId::of::<Connect>(),
        State::Disconnected,
        move |state| {
            let data = data.clone();
            let config = config.clone();
            async move {
                match state {
                    State::Disconnected => {
                        let check_incoming_interval = config.lock().check_incoming_interval;
                        tokio::time::sleep(check_incoming_interval).await;
                        let client = match Client::new(data, config).await {
                            Ok(c) => c,
                            Err(e) => return (Some(Event::Error(e)), State::Disconnected),
                        };
                        match client.check_incoming_connection().await {
                            Ok(c) => match c {
                                Some(time) => {
                                    (Some(Event::Scheduled(time)), State::Scheduled(client, time))
                                }
                                None => (Some(Event::Checked), State::Disconnected),
                            },
                            Err(e) => (Some(Event::Error(e)), State::Disconnected),
                        }
                    }
                    State::Scheduled(client, time) => {
                        if let Err(e) = sleep_till(time).await {
                            return (Some(Event::Error(e)), State::Disconnected);
                        }

                        match client.receive_incoming_connection().await {
                            Ok(conn) => (Some(Event::Connected), State::Connected(conn)),
                            Err(e) => (Some(Event::Error(e)), State::Disconnected),
                        }
                    }
                    State::Connected(conn) => (
                        match conn.handle().await {
                            Ok(_) => Some(Event::Complete),
                            Err(e) => Some(Event::Error(e)),
                        },
                        State::Disconnected,
                    ),
                }
            }
        },
    )
}

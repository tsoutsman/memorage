use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::index::Index,
    net::{
        peer::{sleep_till, OutgoingConnection},
        Client,
    },
    persistent::{config::Config, data::Data, Persistent},
    Result,
};
use memorage_core::time::OffsetDateTime;

use tokio::sync::mpsc::channel;
use tracing::{debug, error, info, trace};

pub async fn daemon(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<!> {
    let config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        let server_address = &mut config.lock().server_address;
        *server_address = vec![server];
    }

    let (incoming_tx, mut incoming_rx) = channel::<IncomingEvent>(10);
    let incoming_config = config.clone();
    let incoming_data = data.clone();

    let _incoming = tokio::spawn(async move {
        let config = incoming_config;
        let data = incoming_data;
        loop {
            let result: Result<()> = try {
                let client = Client::new(data.clone(), config.clone()).await?;
                match client.check_incoming_connection().await? {
                    Some(time) => {
                        let _ = incoming_tx.send(IncomingEvent::Scheduled(time)).await;
                        sleep_till(time).await?;
                        let _ = incoming_tx.send(IncomingEvent::Connecting).await;
                        let conn = client.receive_incoming_connection().await?;
                        let _ = incoming_tx.send(IncomingEvent::Connected).await;
                        conn.handle().await?;
                    }
                    None => {
                        let _ = incoming_tx.send(IncomingEvent::Checked).await;
                    }
                }
            };

            match result {
                Ok(_) => {
                    let _ = incoming_tx.send(IncomingEvent::Complete).await;
                }
                Err(e) => {
                    let _ = incoming_tx.send(IncomingEvent::Error(e)).await;
                }
            };

            let check_incoming_interval = config.lock().check_incoming_interval;
            tokio::time::sleep(check_incoming_interval).await;
        }
    });

    let (outgoing_tx, mut outgoing_rx) = channel::<OutgoingEvent>(10);
    let outgoing_config = config.clone();
    let outgoing_data = data.clone();

    let _outgoing = tokio::spawn(async move {
        let config = outgoing_config;
        let data = outgoing_data;
        loop {
            let result: Result<()> = try {
                let client = Client::new(data.clone(), config.clone()).await?;
                let time = client.schedule_outgoing_connection().await?;
                let _ = outgoing_tx.send(OutgoingEvent::Scheduled(time)).await;

                let backup_path_clone = config.lock().backup_path.clone();
                let new_index_handle = tokio::spawn(async move {
                    // TODO: Race conditions?
                    Index::from_directory(backup_path_clone).await
                });

                sleep_till(time).await?;
                let _ = outgoing_tx.send(OutgoingEvent::Connecting).await;
                let mut conn = client.create_outgoing_connection().await?;

                async fn indefinite_ping(connection: &mut OutgoingConnection) -> ! {
                    loop {
                        // TODO: The select statement could drop indefinite_ping during the
                        // ping, which may result in a write error on the peer if
                        // indefinite_ping gets dropped after transmitting a ping but before
                        // receiving a response.
                        let result = connection.ping().await;

                        trace!(?result, "pinged peer");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
                let new_index = tokio::select! {
                    // Biased mode first checks if the new index has already been created
                    // before beginning to ping.
                    biased;
                    new_index = new_index_handle => new_index??,
                    // indefinite_ping will keep pinging the peer to keep the connection
                    // open until the local index has been created. Index::new() is just
                    // there to satisfy the type checker.
                    _ = indefinite_ping(&mut conn) => Index::new(),
                };
                debug!("new index created");

                let _ = outgoing_tx.send(OutgoingEvent::Connected).await;
                conn.backup(&new_index).await?;
                let _ = outgoing_tx.send(OutgoingEvent::Complete).await;
            };

            match result {
                Ok(_) => {
                    let _ = outgoing_tx.send(OutgoingEvent::Complete).await;
                }
                Err(e) => {
                    let _ = outgoing_tx.send(OutgoingEvent::Error(e)).await;
                }
            };

            let schedule_outgoing_interval = config.lock().schedule_outgoing_interval;
            tokio::time::sleep(schedule_outgoing_interval).await;
        }
    });

    loop {
        tokio::select! {
            event = incoming_rx.recv() => {
                let event = event.expect("incoming handler dropped sender");
                match event {
                    IncomingEvent::Checked => info!("checked server for connection requests"),
                    IncomingEvent::Scheduled(time) => {
                        info!("scheduled to receieve backup from peer at {time}")
                    }
                    IncomingEvent::Connecting => info!("connecting to peer to receive backup"),
                    IncomingEvent::Connected => info!("connected to peer - ready to recieve backup"),
                    IncomingEvent::Complete => info!("sucessfuly receieved backup from peer"),
                    IncomingEvent::Error(error) => {
                        error!("error on incoming connection handler: {error}")
                    }
                }
            },
            event = outgoing_rx.recv() => {
                let event = event.expect("outgoing handler dropped sender");
                match event {
                    OutgoingEvent::Scheduled(time) => info!("scheduled to backup to peer at {time}"),
                    OutgoingEvent::Connecting => info!("connecting to peer to transfer backup"),
                    OutgoingEvent::Connected => info!("connected to peer - ready to transfer backup"),
                    OutgoingEvent::Complete => info!("succesfully transferred backup"),
                    OutgoingEvent::Error(error) => error!("error on outgoing connection handler: {error}"),
                }
            }
        }
    }
}

enum IncomingEvent {
    Checked,
    Scheduled(OffsetDateTime),
    Connecting,
    Connected,
    Complete,
    Error(memorage_client::Error),
}

enum OutgoingEvent {
    Scheduled(OffsetDateTime),
    Connecting,
    Connected,
    Complete,
    Error(memorage_client::Error),
}

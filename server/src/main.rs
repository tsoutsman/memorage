use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (channels, _handles) = server::setup();

    let listener = TcpListener::bind("0.0.0.0:1117").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tracing::event!(tracing::Level::INFO, "connection from {:#?}", addr);
        tokio::spawn(server::handle_request(socket, addr, channels.clone()));
    }

    #[allow(unreachable_code)]
    {
        _handles.join().await?;
        Ok(())
    }
}

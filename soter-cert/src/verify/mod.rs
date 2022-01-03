mod schema;

use rustls::{
    client::{ServerCertVerified, ServerName},
    Certificate,
};

// TODO wrap in Arc?
pub struct ServerCertVerifier<T>(pub T)
where
    T: Fn() -> Result<ServerCertVerified, rustls::Error> + Send + Sync;

impl<T> rustls::client::ServerCertVerifier for ServerCertVerifier<T>
where
    T: Fn() -> Result<ServerCertVerified, rustls::Error> + Send + Sync,
{
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        // :)
        // I feel like this isn't safe
        (self.0)()
    }
}

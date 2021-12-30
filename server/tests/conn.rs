mod util;

use lib::cs::{
    key::VerifiablePublicKey,
    protocol::{request, response},
};
use util::{ADDR_1, ADDR_2, KEYPAIR_1, KEYPAIR_2};

#[tokio::test]
async fn basic() {
    let (channels, _handles) = server::setup();

    let request = request::GetSigningBytes;
    let response = util::request(request, *ADDR_1, channels.clone()).await;
    let signing_bytes = response.unwrap().0;

    let initiator_key = VerifiablePublicKey::new(&KEYPAIR_1, &signing_bytes);
    let request = request::RequestConnection {
        initiator_key,
        target_key: KEYPAIR_2.public,
    };
    let response = util::request(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(response::RequestConnection));

    let request = request::Ping;
    let response = util::request(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(None)));

    let target_key = VerifiablePublicKey::new(&KEYPAIR_2, &signing_bytes);
    let request = request::CheckConnection(target_key);
    let response = util::request(request, *ADDR_2, channels.clone()).await;
    assert_eq!(response, Ok(response::CheckConnection(Some(*ADDR_1))));

    let request = request::Ping;
    let response = util::request(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(Some(*ADDR_2))))
}

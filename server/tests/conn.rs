mod util;

use lib::cs::{
    key::VerifiablePublicKey,
    protocol::{
        error::Error,
        request::Request,
        response::{CheckConnection, GetSigningBytes, Ping, RequestConnection},
    },
};
use util::{ADDR_1, ADDR_2, KEYPAIR_1, KEYPAIR_2};

#[tokio::test]
async fn basic() {
    let (channels, _handles) = server::setup();

    let request = Request::GetSigningBytes;
    let response: Result<GetSigningBytes, Error> =
        util::request(request, *ADDR_1, channels.clone()).await;
    let signing_bytes = response.unwrap().0;

    let initiator_key = VerifiablePublicKey::new(&KEYPAIR_1, &signing_bytes);
    let request = Request::RequestConnection {
        initiator_key,
        target_key: KEYPAIR_2.public,
    };
    let response: Result<RequestConnection, Error> =
        util::request(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(RequestConnection));

    let request = Request::Ping;
    let response = util::request::<Ping>(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(Ping(None)));

    let target_key = VerifiablePublicKey::new(&KEYPAIR_2, &signing_bytes);
    let request = Request::CheckConnection(target_key);
    let response = util::request::<CheckConnection>(request, *ADDR_2, channels.clone()).await;
    assert_eq!(response, Ok(CheckConnection(Some(*ADDR_1))));

    let request = Request::Ping;
    let response = util::request::<Ping>(request, *ADDR_1, channels.clone()).await;
    assert_eq!(response, Ok(Ping(Some(*ADDR_2))))
}

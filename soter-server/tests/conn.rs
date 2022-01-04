mod util;

use soter_cs::{request, response};
use util::{ID_1, ID_2};

#[tokio::test]
async fn basic() {
    let (channels, _handles) = soter_server::setup();

    let request = request::RequestConnection(ID_2.public_key);
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Ok(response::RequestConnection));

    let request = request::Ping;
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(None)));

    let request = request::CheckConnection;
    let response = util::request(request, &ID_2, channels.clone()).await;
    assert_eq!(response, Ok(response::CheckConnection(Some(ID_1.address))));

    let request = request::Ping;
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(Some(ID_2.address))))
}

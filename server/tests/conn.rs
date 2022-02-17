mod util;

use util::{ID_1, ID_2};

use memorage_core::time::OffsetDateTime;
use memorage_cs::{request, response, Error};

#[tokio::test]
async fn basic() {
    memorage_server::setup_logger();
    let (channels, _handles) = memorage_server::setup();

    let time = OffsetDateTime::now_utc();
    let request = request::RequestConnection {
        target: ID_2.public_key,
        time,
    };
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Ok(response::RequestConnection));

    let request = request::Ping(ID_2.public_key);
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Err(Error::NoData));

    let request = request::CheckConnection;
    let response = util::request(request, &ID_2, channels.clone()).await;
    assert_eq!(
        response,
        Ok(response::CheckConnection {
            initiator: ID_1.public_key,
            time
        })
    );

    let request = request::Ping(ID_2.public_key);
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Err(Error::NoData));

    let request = request::Ping(ID_1.public_key);
    let response = util::request(request, &ID_2, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(ID_1.address)));

    let request = request::Ping(ID_1.public_key);
    let response = util::request(request, &ID_2, channels.clone()).await;
    assert_eq!(response, Err(Error::NoData));

    let request = request::Ping(ID_2.public_key);
    let response = util::request(request, &ID_1, channels.clone()).await;
    assert_eq!(response, Ok(response::Ping(ID_2.address)));
}

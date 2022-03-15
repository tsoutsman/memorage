mod util;

use util::{ID_1, ID_2};

use memorage_cs::{request, response, Error, PairingCode};

#[tokio::test]
async fn correct_code() {
    let (channels, _handles) = memorage_server::setup();

    let request = request::Register;
    let response = util::request(request, &ID_1, channels.clone()).await;

    let code = response.unwrap().0;
    let request = request::GetKey(code);
    let response = util::request(request.clone(), &ID_2, channels.clone()).await;

    assert_eq!(response, Ok(response::GetKey(ID_1.public_key)));

    // Make sure server deletes code after it has been accessed
    let response = util::request(request.clone(), &ID_2, channels.clone()).await;

    assert_eq!(response, Err(Error::NoData));
}

#[tokio::test]
async fn incorrect_code() {
    let (channels, _handles) = memorage_server::setup();

    let request = request::Register;
    let response = util::request(request, &ID_1, channels.clone()).await;

    // Make sure there was no error adding the code.
    response.unwrap();

    let request = request::GetKey(PairingCode::new());
    let response = util::request(request, &ID_2, channels.clone()).await;

    assert_eq!(response, Err(Error::NoData));
}

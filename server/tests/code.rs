mod util;

use util::{ADDR_1, KEYPAIR_1};

use lib::cs::{
    protocol::{error::Error, request, response},
    Code,
};

#[tokio::test]
async fn correct_code() {
    let (channels, _handles) = server::setup();
    let public_key = KEYPAIR_1.public;

    let request = request::Register(public_key);
    let response = util::request(request, *ADDR_1, channels.clone()).await;

    let code = response.unwrap().0;
    let request = request::GetKey(code);
    let response = util::request(request.clone(), *ADDR_1, channels.clone()).await;

    assert_eq!(response, Ok(response::GetKey(public_key)));

    // Make sure server deletes code after it has been accessed
    let response = util::request(request.clone(), *ADDR_1, channels.clone()).await;

    assert_eq!(response, Err(Error::InvalidCode));
}

#[tokio::test]
async fn incorrect_code() {
    let (channels, _handles) = server::setup();
    let public_key = KEYPAIR_1.public;

    let request = request::Register(public_key);
    let response = util::request(request, *ADDR_1, channels.clone()).await;

    // Make sure there was no error adding the code.
    response.unwrap();

    let request = request::GetKey(Code::new());
    let response = util::request(request, *ADDR_1, channels.clone()).await;

    assert_eq!(response, Err(Error::InvalidCode));
}

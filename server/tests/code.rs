mod util;

use util::{ADDR_1, KEYPAIR_1};

use lib::cs::{
    protocol::{
        error::Error,
        request::Request,
        response::{GetKey, Register},
    },
    Code,
};

#[tokio::test]
async fn correct_code() {
    let (channels, _handles) = server::setup();
    let public_key = KEYPAIR_1.public;

    let request = Request::Register(public_key);
    let response: Result<Register, Error> = util::request(request, *ADDR_1, channels.clone()).await;

    let code = response.unwrap().0;
    let request = Request::GetKey(code);
    let response: Result<GetKey, Error> =
        util::request(request.clone(), *ADDR_1, channels.clone()).await;

    assert_eq!(response.unwrap().0, public_key);

    // Make sure server deletes code after it has been accessed
    let response: Result<GetKey, Error> =
        util::request(request.clone(), *ADDR_1, channels.clone()).await;

    assert_eq!(response, Err(Error::InvalidCode));
}

#[tokio::test]
async fn incorrect_code() {
    let (channels, _handles) = server::setup();
    let public_key = KEYPAIR_1.public;

    let request = Request::Register(public_key);
    let response: Result<Register, Error> = util::request(request, *ADDR_1, channels.clone()).await;

    // Make sure there was no error adding the code.
    response.unwrap();

    let request = Request::GetKey(Code::new());
    let response: Result<GetKey, Error> = util::request(request, *ADDR_1, channels.clone()).await;

    assert_eq!(response, Err(Error::InvalidCode));
}

mod util;

use util::{ADDR_1, KEY_1};

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

    let request = Request::Register(*KEY_1);
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, *ADDR_1, channels.clone()).await;
    let response: Result<Register, Error> = bincode::deserialize(&buffer.output()).unwrap();

    let code = response.unwrap().0;
    let request = Request::GetKey(code);
    let mut buffer = util::MockRequest::from(request.clone());

    server::handle_request(&mut buffer, *ADDR_1, channels.clone()).await;
    let response: Result<GetKey, Error> = bincode::deserialize(&buffer.output()).unwrap();

    assert_eq!(response.unwrap().0, *KEY_1);

    // Make sure server deletes code after it has been accessed
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, *ADDR_1, channels.clone()).await;
    let response: Result<GetKey, Error> = bincode::deserialize(&buffer.output()).unwrap();

    assert_eq!(response, Err(Error::InvalidCode));
}

#[tokio::test]
async fn incorrect_code() {
    let (channels, _handles) = server::setup();
    let request = Request::Register(*KEY_1);
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, *ADDR_1, channels.clone()).await;
    let response: Result<Register, Error> = bincode::deserialize(&buffer.output()).unwrap();

    // Make sure there was no error adding the code.
    response.unwrap();

    let request = Request::GetKey(Code::new());
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, *ADDR_1, channels.clone()).await;
    let response: Result<GetKey, Error> = bincode::deserialize(&buffer.output()).unwrap();

    assert_eq!(response, Err(Error::InvalidCode));
}

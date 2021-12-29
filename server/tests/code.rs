mod util;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use util::KEY_1;

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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 8080);

    let request = Request::Register(*KEY_1);
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, addr, channels.clone()).await;
    let response: Result<Register, Error> = bincode::deserialize(&buffer.output()).unwrap();

    let code = response.unwrap().0;
    let request = Request::GetKey(code);
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, addr, channels.clone()).await;
    let response: Result<GetKey, Error> = bincode::deserialize(&buffer.output()).unwrap();

    assert_eq!(response.unwrap().0, *KEY_1);
}

#[tokio::test]
async fn incorrect_code() {
    let (channels, _handles) = server::setup();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 8080);
    let request = Request::Register(*KEY_1);
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, addr, channels.clone()).await;
    let response: Result<Register, Error> = bincode::deserialize(&buffer.output()).unwrap();

    // Make sure there was no error adding the code.
    response.unwrap();

    let request = Request::GetKey(Code::new());
    let mut buffer = util::MockRequest::from(request);

    server::handle_request(&mut buffer, addr, channels.clone()).await;
    let response: Result<GetKey, Error> = bincode::deserialize(&buffer.output()).unwrap();

    assert_eq!(response, Err(Error::InvalidCode));
}

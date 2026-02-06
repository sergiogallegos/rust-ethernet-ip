use rust_ethernet_ip::error::EtherNetIpError;
use rust_ethernet_ip::EipClient;

#[tokio::test]
async fn connect_invalid_format_fails_fast() {
    let err = EipClient::connect("bad-address").await.unwrap_err();
    match err {
        EtherNetIpError::Protocol(message) => {
            assert!(message.contains("Invalid address format"));
        }
        other => panic!("expected Protocol error, got {other:?}"),
    }
}

#[tokio::test]
async fn connect_refused_is_io_error() {
    let err = EipClient::connect("127.0.0.1:1").await.unwrap_err();
    match err {
        EtherNetIpError::Io(_) => {}
        other => panic!("expected Io error, got {other:?}"),
    }
}

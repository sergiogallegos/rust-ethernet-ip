use rust_ethernet_ip::EipClient;
use rust_ethernet_ip::error::EtherNetIpError;
use std::io;
use std::time::Duration;

#[test]
fn error_display_messages_are_informative() {
    let errors = vec![
        (
            EtherNetIpError::Protocol("bad packet".to_string()),
            "Protocol error",
        ),
        (
            EtherNetIpError::TagNotFound("MyTag".to_string()),
            "Tag not found",
        ),
        (
            EtherNetIpError::DataTypeMismatch {
                expected: "INT".to_string(),
                actual: "DINT".to_string(),
            },
            "Data type mismatch",
        ),
        (
            EtherNetIpError::WriteError {
                status: 0x10,
                message: "write failed".to_string(),
            },
            "Write error",
        ),
        (
            EtherNetIpError::ReadError {
                status: 0x05,
                message: "read failed".to_string(),
            },
            "Read error",
        ),
        (
            EtherNetIpError::InvalidResponse {
                reason: "short".to_string(),
            },
            "Invalid response",
        ),
        (
            EtherNetIpError::Timeout(Duration::from_secs(1)),
            "Operation timed out",
        ),
        (EtherNetIpError::Udt("bad udt".to_string()), "UDT error"),
        (
            EtherNetIpError::Connection("no session".to_string()),
            "Connection error",
        ),
        (
            EtherNetIpError::ConnectionLost("network closed".to_string()),
            "Connection lost",
        ),
        (
            EtherNetIpError::CipError {
                code: 0x07,
                message: "connection lost".to_string(),
            },
            "CIP error",
        ),
        (
            EtherNetIpError::StringTooLong {
                max_length: 82,
                actual_length: 83,
            },
            "String too long",
        ),
        (
            EtherNetIpError::InvalidString {
                reason: "non-ascii".to_string(),
            },
            "Invalid string",
        ),
        (
            EtherNetIpError::StringWriteError {
                status: 0x07,
                message: "string write failed".to_string(),
            },
            "String write failed",
        ),
        (
            EtherNetIpError::StringReadError {
                status: 0x08,
                message: "string read failed".to_string(),
            },
            "String read failed",
        ),
        (
            EtherNetIpError::InvalidStringResponse {
                reason: "bad format".to_string(),
            },
            "Invalid string response",
        ),
        (EtherNetIpError::Tag("bad tag".to_string()), "Tag error"),
        (
            EtherNetIpError::Permission("read denied".to_string()),
            "Permission denied",
        ),
        (
            EtherNetIpError::Utf8(String::from_utf8(vec![0xff]).unwrap_err()),
            "UTF-8 error",
        ),
        (EtherNetIpError::Other("misc".to_string()), "Other error"),
        (
            EtherNetIpError::Subscription("sub failed".to_string()),
            "Subscription error",
        ),
    ];

    for (err, expected) in errors {
        let msg = err.to_string();
        assert!(
            msg.contains(expected),
            "expected '{expected}' in error message '{msg}'"
        );
    }
}

#[tokio::test]
async fn connect_invalid_address_returns_protocol_error() {
    let err = EipClient::connect("not-an-ip").await.unwrap_err();
    match err {
        EtherNetIpError::Protocol(message) => {
            assert!(
                message.contains("Invalid address format"),
                "unexpected protocol message: {message}"
            );
        }
        other => panic!("expected Protocol error, got {other:?}"),
    }
}

#[tokio::test]
async fn connect_refused_returns_io_error() {
    let err = EipClient::connect("127.0.0.1:1").await.unwrap_err();
    match err {
        EtherNetIpError::Io(io_err) => {
            assert_ne!(io_err.kind(), io::ErrorKind::InvalidInput);
        }
        other => panic!("expected Io error, got {other:?}"),
    }
}

#[test]
fn is_retriable_identifies_retriable_errors() {
    use std::time::Duration;

    assert!(EtherNetIpError::Timeout(Duration::from_secs(1)).is_retriable());
    assert!(EtherNetIpError::Connection("lost".to_string()).is_retriable());
    assert!(EtherNetIpError::ConnectionLost("closed".to_string()).is_retriable());
    assert!(
        EtherNetIpError::Io(io::Error::new(io::ErrorKind::TimedOut, "timed out")).is_retriable()
    );

    assert!(!EtherNetIpError::TagNotFound("X".to_string()).is_retriable());
    assert!(!EtherNetIpError::Protocol("bad".to_string()).is_retriable());
    assert!(
        !EtherNetIpError::CipError {
            code: 0x16,
            message: "object does not exist".to_string(),
        }
        .is_retriable()
    );
}

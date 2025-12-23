use serde_test::{assert_tokens, Configure, Token};
use std::net;

#[macro_use]
#[allow(unused_macros)]
mod macros;

#[test]
fn ip_addr_roundtrip() {
    assert_tokens(
        &net::IpAddr::from(*b"1234").compact(),
        &seq![
            Token::NewtypeVariant {
                name: "IpAddr",
                variant: "V4"
            },
            Token::Tuple { len: 4 },
            b"1234".iter().copied().map(Token::U8),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_roundtrip() {
    assert_tokens(
        &net::SocketAddr::from((*b"1234567890123456", 1234)).compact(),
        &seq![
            Token::NewtypeVariant {
                name: "SocketAddr",
                variant: "V6"
            },
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            b"1234567890123456".iter().copied().map(Token::U8),
            Token::TupleEnd,
            Token::U16(1234),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn ipv4_addr_roundtrip() {
    assert_tokens(
        &net::Ipv4Addr::new(192, 168, 1, 1).compact(),
        &seq![
            Token::Tuple { len: 4 },
            Token::U8(192),
            Token::U8(168),
            Token::U8(1),
            Token::U8(1),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn ipv6_addr_roundtrip() {
    assert_tokens(
        &net::Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334)
            .compact(),
        &seq![
            Token::Tuple { len: 16 },
            Token::U8(0x20),
            Token::U8(0x01),
            Token::U8(0x0d),
            Token::U8(0xb8),
            Token::U8(0x85),
            Token::U8(0xa3),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x8a),
            Token::U8(0x2e),
            Token::U8(0x03),
            Token::U8(0x70),
            Token::U8(0x73),
            Token::U8(0x34),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn ip_addr_v6_roundtrip() {
    assert_tokens(
        &net::IpAddr::from(net::Ipv6Addr::new(
            0xfe80, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0001,
        ))
        .compact(),
        &seq![
            Token::NewtypeVariant {
                name: "IpAddr",
                variant: "V6"
            },
            Token::Tuple { len: 16 },
            Token::U8(0xfe),
            Token::U8(0x80),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x01),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_v4_roundtrip() {
    assert_tokens(
        &net::SocketAddrV4::new(net::Ipv4Addr::new(127, 0, 0, 1), 8080).compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 4 },
            Token::U8(127),
            Token::U8(0),
            Token::U8(0),
            Token::U8(1),
            Token::TupleEnd,
            Token::U16(8080),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_v6_roundtrip() {
    assert_tokens(
        &net::SocketAddrV6::new(
            net::Ipv6Addr::new(0x2001, 0x0db8, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0001),
            443,
            0,
            0,
        )
        .compact(),
        &seq![
            Token::Tuple { len: 2 },
            Token::Tuple { len: 16 },
            Token::U8(0x20),
            Token::U8(0x01),
            Token::U8(0x0d),
            Token::U8(0xb8),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x00),
            Token::U8(0x01),
            Token::TupleEnd,
            Token::U16(443),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn socket_addr_v4_variant_roundtrip() {
    assert_tokens(
        &net::SocketAddr::V4(net::SocketAddrV4::new(net::Ipv4Addr::new(10, 0, 0, 1), 3000))
            .compact(),
        &seq![
            Token::NewtypeVariant {
                name: "SocketAddr",
                variant: "V4"
            },
            Token::Tuple { len: 2 },
            Token::Tuple { len: 4 },
            Token::U8(10),
            Token::U8(0),
            Token::U8(0),
            Token::U8(1),
            Token::TupleEnd,
            Token::U16(3000),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn ipv4_addr_loopback_roundtrip() {
    assert_tokens(
        &net::Ipv4Addr::LOCALHOST.compact(),
        &seq![
            Token::Tuple { len: 4 },
            Token::U8(127),
            Token::U8(0),
            Token::U8(0),
            Token::U8(1),
            Token::TupleEnd,
        ],
    );
}

#[test]
fn ipv6_addr_loopback_roundtrip() {
    assert_tokens(
        &net::Ipv6Addr::LOCALHOST.compact(),
        &seq![
            Token::Tuple { len: 16 },
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U8(1),
            Token::TupleEnd,
        ],
    );
}

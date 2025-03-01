// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This module provides a set of integration tests and a function to run the
//! tests against a Segcache instance. This allows us to run the same test suite
//! for multiple server configurations.

use logger::*;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub fn tests() {
    debug!("beginning tests");
    println!();

    test(
        "cas not found (key: 0)",
        &[("cas 0 0 0 1 1\r\n0\r\n", Some("NOT_FOUND\r\n"))],
    );
    test("get empty (key: 0)", &[("get 0\r\n", Some("END\r\n"))]);
    test("gets empty (key: 0)", &[("gets 0\r\n", Some("END\r\n"))]);
    test(
        "cas not found (key: 0)",
        &[("cas 0 0 0 1 0\r\n0\r\n", Some("NOT_FOUND\r\n"))],
    );
    test(
        "set value (key: 0)",
        &[("set 0 0 0 1\r\n1\r\n", Some("STORED\r\n"))],
    );
    test(
        "get value (key: 0)",
        &[("get 0\r\n", Some("VALUE 0 0 1\r\n1\r\nEND\r\n"))],
    );
    test(
        "gets value (key: 0)",
        &[("gets 0\r\n", Some("VALUE 0 0 1 1\r\n1\r\nEND\r\n"))],
    );
    test(
        "cas fail (key: 0)",
        &[("cas 0 0 0 1 0\r\n1\r\n", Some("EXISTS\r\n"))],
    );
    test(
        "cas success (key: 0)",
        &[("cas 0 0 0 1 1\r\n1\r\n", Some("STORED\r\n"))],
    );
    test(
        "add value (key: 0)",
        &[("add 0 0 0 1\r\n2\r\n", Some("NOT_STORED\r\n"))],
    );
    test(
        "add value (key: 1)",
        &[("add 1 0 0 1\r\n2\r\n", Some("STORED\r\n"))],
    );
    test(
        "get value (key: 0)",
        &[("get 0\r\n", Some("VALUE 0 0 1\r\n1\r\nEND\r\n"))],
    );
    test(
        "get value (key: 1)",
        &[("get 1\r\n", Some("VALUE 1 0 1\r\n2\r\nEND\r\n"))],
    );
    test(
        "replace value (key: 1)",
        &[("replace 1 0 0 1\r\n3\r\n", Some("STORED\r\n"))],
    );
    test(
        "replace value (key: 2)",
        &[("replace 2 0 0 1\r\n2\r\n", Some("NOT_STORED\r\n"))],
    );
    test(
        "get value (key: 1)",
        &[("get 1\r\n", Some("VALUE 1 0 1\r\n3\r\nEND\r\n"))],
    );
    test("get value (key: 2)", &[("get 2\r\n", Some("END\r\n"))]);

    // test storing and retrieving flags
    test(
        "set value (key: 3)",
        &[("set 3 42 0 1\r\n1\r\n", Some("STORED\r\n"))],
    );
    test(
        "get value (key: 3)",
        &[("get 3\r\n", Some("VALUE 3 42 1\r\n1\r\nEND\r\n"))],
    );

    // test pipelined commands
    test(
        "pipelined get (key: 4 depth: 2)",
        &[("get 4\r\nget 4\r\n", Some("END\r\nEND\r\n"))],
    );
    test(
        "pipelined get and invalid (key 4, depth 2)",
        &[("get 4\r\n ", Some("END\r\n"))],
    );
    test(
        "pipelined get and add (key 4, depth 2)",
        &[("get 4 \r\nadd 4 0 0 1\r\n1\r\n", Some("END\r\nSTORED\r\n"))],
    );
    test(
        "pipelined get and set (key 5, depth 2)",
        &[("get 5 \r\nset 5 0 0 1 \r\n1\r\n", Some("END\r\nSTORED\r\n"))],
    );
    test(
        "pipelined set and get (key 6, depth 3)",
        &[(
            "set 6 0 0 2 \r\nhi\r\nset 6 0 0 6\r\nhello!\r\nget 6 \r\n",
            Some("STORED\r\nSTORED\r\nVALUE 6 0 6\r\nhello!\r\nEND\r\n"),
        )],
    );

    // test increment
    test("incr (key: 9)", &[("incr 9 1\r\n", Some("NOT_FOUND\r\n"))]);
    test(
        "set value (key: 9)",
        &[("set 9 0 0 1\r\n0\r\n", Some("STORED\r\n"))],
    );
    test("incr (key: 9)", &[("incr 9 1\r\n", Some("1\r\n"))]);
    test("incr (key: 9)", &[("incr 9 2\r\n", Some("3\r\n"))]);
    test(
        "incr (key: 9)",
        &[(&format!("incr 9 {}\r\n", u64::MAX), Some("2\r\n"))],
    );
    test(
        "set value (key: 9)",
        &[("set 9 0 0 1\r\na\r\n", Some("STORED\r\n"))],
    );
    test("incr (key: 9)", &[("incr 9 1\r\n", Some("ERROR\r\n"))]);

    // test decrement
    test(
        "decr (key: 10)",
        &[("decr 10 1\r\n", Some("NOT_FOUND\r\n"))],
    );
    test(
        "set value (key: 10)",
        &[("set 10 0 0 2\r\n10\r\n", Some("STORED\r\n"))],
    );
    test("decr (key: 10)", &[("decr 10 1\r\n", Some("9\r\n"))]);
    test("decr (key: 10)", &[("decr 10 2\r\n", Some("7\r\n"))]);
    test("decr (key: 10)", &[("decr 10 8\r\n", Some("0\r\n"))]);
    test(
        "set value (key: 10)",
        &[("set 10 0 0 1\r\na\r\n", Some("STORED\r\n"))],
    );
    test("decr (key: 10)", &[("decr 10 1\r\n", Some("ERROR\r\n"))]);

    // test unsupported commands
    test(
        "append (key: 7)",
        &[("append 7 0 0 1\r\n0\r\n", Some("ERROR\r\n"))],
    );
    test(
        "prepend (key: 8)",
        &[("prepend 8 0 0 1\r\n0\r\n", Some("ERROR\r\n"))],
    );

    std::thread::sleep(Duration::from_millis(500));
}

// opens a new connection, operating on request + response pairs from the
// provided data.
fn test(name: &str, data: &[(&str, Option<&str>)]) {
    info!("testing: {}", name);
    debug!("connecting to server");
    let mut stream = TcpStream::connect("127.0.0.1:12321").expect("failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("failed to set read timeout");
    stream
        .set_write_timeout(Some(Duration::from_millis(250)))
        .expect("failed to set write timeout");

    debug!("sending request");
    for (request, response) in data {
        match stream.write(request.as_bytes()) {
            Ok(bytes) => {
                if bytes == request.len() {
                    debug!("full request sent");
                } else {
                    error!("incomplete write");
                    panic!("status: failed\n");
                }
            }
            Err(_) => {
                error!("error sending request");
                panic!("status: failed\n");
            }
        }

        std::thread::sleep(Duration::from_millis(10));
        let mut buf = vec![0; 4096];

        if let Some(response) = response {
            if stream.read(&mut buf).is_err() {
                std::thread::sleep(Duration::from_millis(500));
                panic!("error reading response");
            } else if response.as_bytes() != &buf[0..response.len()] {
                error!("expected: {:?}", response.as_bytes());
                error!("received: {:?}", &buf[0..response.len()]);
                std::thread::sleep(Duration::from_millis(500));
                panic!("status: failed\n");
            } else {
                debug!("correct response");
            }
            assert_eq!(response.as_bytes(), &buf[0..response.len()]);
        } else if let Err(e) = stream.read(&mut buf) {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                debug!("got no response");
            } else {
                error!("error reading response");
                std::thread::sleep(Duration::from_millis(500));
                panic!("status: failed\n");
            }
        } else {
            error!("expected no response");
            std::thread::sleep(Duration::from_millis(500));
            panic!("status: failed\n");
        }

        if data.len() > 1 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    info!("status: passed\n");
}

pub fn admin_tests() {
    debug!("beginning admin tests");
    println!();

    admin_test(
        "version",
        &[(
            "version\r\n",
            Some(&format!("VERSION {}\r\n", env!("CARGO_PKG_VERSION"))),
        )],
    );
}

// opens a new connection to the admin port, sends a request, and checks the response.
fn admin_test(name: &str, data: &[(&str, Option<&str>)]) {
    info!("testing: {}", name);
    debug!("connecting to server");
    let mut stream = TcpStream::connect("127.0.0.1:9999").expect("failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("failed to set read timeout");
    stream
        .set_write_timeout(Some(Duration::from_millis(250)))
        .expect("failed to set write timeout");

    debug!("sending request");
    for (request, response) in data {
        match stream.write(request.as_bytes()) {
            Ok(bytes) => {
                if bytes == request.len() {
                    debug!("full request sent");
                } else {
                    error!("incomplete write");
                    panic!("status: failed\n");
                }
            }
            Err(_) => {
                error!("error sending request");
                panic!("status: failed\n");
            }
        }

        std::thread::sleep(Duration::from_millis(10));
        let mut buf = vec![0; 4096];

        if let Some(response) = response {
            if stream.read(&mut buf).is_err() {
                std::thread::sleep(Duration::from_millis(500));
                panic!("error reading response");
            } else if response.as_bytes() != &buf[0..response.len()] {
                error!("expected: {:?}", response.as_bytes());
                error!("received: {:?}", &buf[0..response.len()]);
                std::thread::sleep(Duration::from_millis(500));
                panic!("status: failed\n");
            } else {
                debug!("correct response");
            }
            assert_eq!(response.as_bytes(), &buf[0..response.len()]);
        } else if let Err(e) = stream.read(&mut buf) {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                debug!("got no response");
            } else {
                error!("error reading response");
                std::thread::sleep(Duration::from_millis(500));
                panic!("status: failed\n");
            }
        } else {
            error!("expected no response");
            std::thread::sleep(Duration::from_millis(500));
            panic!("status: failed\n");
        }

        if data.len() > 1 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    info!("status: passed\n");
}

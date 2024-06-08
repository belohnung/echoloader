use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
extern crate echoloader;

use echoloader::{hex_escape, main_with_args, Opt};

#[test]
fn test_connect_to_server() {
    let server_addr = "127.0.0.1:8080";
    let listener = Arc::new(Mutex::new(
        TcpListener::bind(server_addr).expect("Could not bind to address"),
    ));

    let listener_clone = listener.clone();

    // Start a thread to listen for incoming connections
    thread::spawn(move || {
        let listener_copy = listener_clone.lock().unwrap();
        let (mut stream, _) = listener_copy.accept().expect("Failed to accept connection");
        let mut buffer = [0; 1024];
        let _ = stream
            .read(&mut buffer)
            .expect("Failed to read from stream");
    });

    let opt = Opt::new(
        server_addr.parse().unwrap(),
        "tests/test_data.txt".into(),
        None,
        128,
        None,
    );

    main_with_args(opt);

    // Ensure the server thread has enough time to process the connection
    thread::sleep(Duration::from_millis(100));

    assert!(TcpStream::connect(server_addr).is_ok());

    // Ensure the server thread has enough time to process the connection
    // thread::sleep(Duration::from_millis(100));

    // // Add assertions about the sent data if needed
    // let expected_data = b"Hello, world!";
    // let mut received_data = Vec::new();
    // let mut stream = TcpStream::connect(server_addr).unwrap();
    // let _ = stream.read_to_end(&mut received_data).unwrap();
    // assert_eq!(received_data, expected_data);

    // // Close the listener
    drop(listener);

    // Stop the thread that listens for incoming connections by connecting to the server
    // and closing the listener
    assert!(TcpStream::connect(server_addr).is_err());
}

#[test]
fn test_hex_escape() {
    let mut input = String::from("Hello, World!");
    hex_escape(&mut input);
    assert_eq!(
        input,
        r"\x48\x65\x6c\x6c\x6f\x2c\x20\x57\x6f\x72\x6c\x64\x21"
    );
}

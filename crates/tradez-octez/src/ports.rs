use std::net::TcpListener;

pub fn pick_unused_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to an ephemeral port")
        .local_addr()
        .expect("Failed to read local address")
        .port()
}

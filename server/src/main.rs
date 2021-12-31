use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

struct SavedAddr {
    addr: std::net::SocketAddr,
    time: std::time::Instant,
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8888")?;
    let mut unfilled_conns: HashMap<String, SavedAddr> = HashMap::new();

    loop {
        // Receive packet
        let mut buf = [0; 100];
        let (amt, src) = socket.recv_from(&mut buf)?;
        // Cut down buffer to relevant data and convert to String
        let buf = &mut buf[..amt];
        let keyword: String = String::from_utf8(buf.to_vec()).unwrap().trim().to_string();
        // No body? No keyword -- that's not allowed
        if keyword.len() == 0 {
            socket.send_to(b"Keyword invalid", src)?;
            continue;
        }

        // If this is the second source, notify peers of each other
        if unfilled_conns.contains_key(&keyword)
            && unfilled_conns[&keyword].addr != src
            && Instant::now().duration_since(unfilled_conns[&keyword].time)
                < Duration::from_secs(60)
        {
            println!("Second peer for {}: {}", keyword, src.to_string());
            socket.send_to(unfilled_conns[&keyword].addr.to_string().as_bytes(), src)?;
            socket.send_to(src.to_string().as_bytes(), unfilled_conns[&keyword].addr)?;
            unfilled_conns.remove(&keyword);
        } else {
            println!("First peer for {}: {}", keyword, src.to_string());
            unfilled_conns.insert(
                keyword,
                SavedAddr {
                    addr: src,
                    time: Instant::now(),
                },
            );
        }
    }
}

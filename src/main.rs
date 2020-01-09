use std::net::Ipv6Addr;
use std::net::ToSocketAddrs;
use mio::net::UdpSocket;
use mio::{Events, Poll, Interest, Token};
use std::io::ErrorKind;
use std::error::Error;
use fmt_extra::AsciiStr;

fn main() -> Result<(), Box<dyn Error>> {
    let mut sa = (Ipv6Addr::UNSPECIFIED, 514).to_socket_addrs().unwrap();
    let mut u = UdpSocket::bind(sa.next().unwrap()).unwrap();

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    poll.registry().register(&mut u, Token(0), Interest::READABLE)?;

    let mut buf = [0u8;4096];
    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            if event.token() == Token(0) && event.is_readable() {
                loop {
                    let v = u.recv_from(&mut buf);
                    match v {
                        Err(ref e) => {
                            if e.kind() == ErrorKind::WouldBlock {
                                break;
                            }
                            v?;
                        },
                        Ok((sz, src)) => {
                            let p = &buf[..sz];
                            let t = AsciiStr(p);
                            println!("{}: {}", src, t);
                        }
                    }
                }
            }
        }
    }
}

use fmt_extra::AsciiStr;
use futures_lite::AsyncBufReadExt;
use glommio::net::{TcpListener, UdpSocket};
use glommio::{prelude::*, Task};
use std::net::Ipv6Addr;

fn main() {
    let local_ex = LocalExecutor::default();

    local_ex.run(async {
        let tcp_t = Task::local(async {
            let tcp_l = TcpListener::bind((Ipv6Addr::UNSPECIFIED, 514)).unwrap();
            loop {
                let mut s = tcp_l.accept().await.unwrap();
                s.set_buffer_size(2048);
                Task::local(async {
                    let mut s = s;
                    let src = s.peer_addr().unwrap();
                    let mut buf = Vec::new();
                    loop {
                        s.read_until(b'\n', &mut buf).await.unwrap();

                        let is_truncated = if buf.last() != Some(&b'\n') {
                            true
                        } else {
                            buf.pop();
                            false
                        };

                        let t = AsciiStr(&buf);

                        println!("{}: {}", src, t);
                        if is_truncated {
                            println!("# warning: previous log message was truncated");
                        }
                    }
                })
                .detach();
            }
        });

        Task::local(async {
            let udp_l = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 514)).unwrap();
            let mut buf = [0u8; 2048];
            loop {
                match udp_l.recv_from(&mut buf).await {
                    Ok((sz, src)) => {
                        let b = &buf[..sz];
                        let t = AsciiStr(b);
                        println!("{}: {}", src, t);
                    }
                    Err(e) => {
                        println!("# error: {}", e);
                    }
                }
            }
        })
        .detach();

        tcp_t.await;
    })
}

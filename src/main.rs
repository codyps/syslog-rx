use fmt_extra::AsciiStr;
use futures_lite::AsyncBufReadExt;
use glommio::net::{TcpListener, UdpSocket};
use glommio::prelude::*;
use listenfd::ListenFd;
use std::net::Ipv6Addr;
use std::os::unix::io::{FromRawFd, IntoRawFd};

fn main() {
    let local_ex = LocalExecutor::default();

    local_ex.run(async {
        let mut listenfd = ListenFd::from_env();

        let tcp_l = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
            unsafe { TcpListener::from_raw_fd(listener.into_raw_fd()) }
        } else {
            TcpListener::bind((Ipv6Addr::UNSPECIFIED, 514)).unwrap()
        };

        let udp_l = if let Some(listener) = listenfd.take_udp_socket(1).unwrap() {
            unsafe { UdpSocket::from_raw_fd(listener.into_raw_fd()) }
        } else {
            UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 514)).unwrap()
        };

        let tcp_t = glommio::spawn_local(async {
            let tcp_l = tcp_l;
            loop {
                let s = tcp_l.accept().await.unwrap().buffered();
                glommio::spawn_local(async {
                    let mut s = s;
                    let src = s.peer_addr().unwrap();
                    let mut buf = Vec::new();
                    loop {
                        if let Err(e) = s.read_until(b'\n', &mut buf).await {
                            println!("{}: read failed, disconnecting: {}", src, e);
                            break;
                        }

                        // split into chunks delineated by b'\n'
                        for m in buf.split_inclusive(|x| *x == b'\n') {
                            let (is_truncated, m) = if m.last() != Some(&b'\n') {
                                (true, m)
                            } else {
                                (false, &m[..m.len() - 1])
                            };

                            let t = AsciiStr(&m);

                            println!("{}: {}", src, t);
                            if is_truncated {
                                println!("# warning: previous log message was truncated");
                            }
                        }
                    }
                })
                .detach();
            }
        });

        glommio::spawn_local(async {
            let udp_l = udp_l;
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

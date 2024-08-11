use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::State> = Default::default();
    let mut nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("Failed to create tun");
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        let _tun_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let tun_protocol = u16::from_be_bytes([buf[2], buf[3]]);

        // lol, just so the compiler thinks I might exit the loop
        if tun_protocol == 0x69 {
            eprintln!("Lol did not expect to get here");
            break;
        }

        if tun_protocol != 0x800 {
            // not ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();

                if proto != etherparse::IpNumber::from(6) {
                    // not tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        let data_start = 4 + iph.slice().len() + tcph.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcph.source_port()),
                                dst: (dst, tcph.destination_port()),
                            })
                            .or_default()
                            .on_packet(&mut nic, &iph, &tcph, &buf[data_start..nbytes])?;
                    }
                    Err(e) => {
                        eprintln!("Ignoring weird tcp packet: {}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("ignoring weird packet: {:?}", e);
                continue;
            }
        };
    }
    Ok(())
}

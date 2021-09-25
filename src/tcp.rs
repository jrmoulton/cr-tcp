use std::convert::TryInto;
pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}

impl Default for State {
    fn default() -> Self {
        State::Closed
    }
}

impl State {
    pub fn on_packet(
        &mut self,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: etherparse::TcpHeaderSlice,
        data: &[u8],
        nic: &mut tun_tap::Iface,
    ) -> std::io::Result<usize> {
        let mut buff = [0u8; 1500];
        match *self {
            State::Closed => {
                return Ok(1);
            }
            State::Listen => {
                if !tcp_header.syn() {
                    return Ok(0);
                }
                let mut syn_ack = etherparse::TcpHeader::new(
                    // The old destination port will become the source port and the old source port
                    // will be the destination port
                    tcp_header.destination_port(),
                    tcp_header.source_port(),
                    0,
                    0,
                );
                syn_ack.syn = true;
                syn_ack.ack = true;
                let ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    255,
                    etherparse::IpTrafficClass::Tcp,
                    ip_header.destination().try_into().unwrap(),
                    ip_header.source().try_into().unwrap(),
                );
                let unwritten = {
                    let mut unwritten = &mut buff[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };
                return nic.send(&buff[..unwritten]);
            }
            State::SynRcvd => Ok(0),
            State::Estab => Ok(0),
        }
    }
}

use std::{io, net};

pub fn get_local_ip() -> io::Result<net::IpAddr> {
    // For the meaning of 0.0.0.0, see 'INADDR_ANY'. Port 0 states that we don't expect any
    // response (RFC 768), so the operating system is allowed to select any ephemeral port.
    let socket = net::UdpSocket::bind(net::SocketAddr::from((net::Ipv4Addr::UNSPECIFIED, 0)))?;

    // Seems to work anyway even with default 'false', so we do this just in case.
    let _ = socket.set_broadcast(true);

    // Obtain an ephemeral port number that the operating system could provide us. It may
    // be outside of the standard IANA range 49152..65535 for ephemeral ports, but we don't
    // check for this because the operating system knows more and better, and we trust it.
    let port = match socket.local_addr() {
        Ok(a) => a.port(),
        Err(_) => 0, // this works too at least on Windows 7, so let's try
    };

    // We're binding socket to INADDR_ANY, so it's necessary to specify a peer in order to
    // obtain something other than 0.0.0.0 from the result of .local_addr() (which actually
    // calls getsockname() under the hood). Since the members of our network are unknown to
    // us, host addresses reserved for broadcasting (see e.g. RFC 1812) are used instead.
    let broadcast: [net::SocketAddr; 5] = [
        // First of all, we try to determine the address of our machine under which it is
        // known to other machines on our LAN. This is the most usable case because it can
        // be used for multiplayer, as well as it should behave as localhost for ourselves.
        ([192, 168, 255, 255], port).into(), // class C private: 192.168.0.0/16
        ([172, 31, 255, 255], port).into(),  // class B private: 172.16.0.0/12
        ([10, 255, 255, 255], port).into(),  // class A private: 10.0.0.0/8
        // See 'INADDR_BROADCAST'. This also handles the virtual network adapters such as
        // ones from VMWare / VirtualBox to imitate the IP lookup based on gethostbyname().
        (net::Ipv4Addr::BROADCAST, port).into(), // limited broadcast: 255.255.255.255/32
        // If all the previous effort has failed (which is pretty unlikely scenario), we
        // give up and try to obtain at least the correct loopback address of this cursed
        // machine. It almost never differs from the common 127.0.0.1, but in theory it is
        // possible, and if we have already got to this place, anything can happen.
        ([127, 255, 255, 255], port).into(), // host loopback: 127.0.0.0/8
    ];

    // Note that there are no real 'connections' in UDP, so what .connect() actually does
    // is a statement to the network driver about the destination address to be used.
    socket.connect(&broadcast[..])?;
    Ok(socket.local_addr()?.ip())
}

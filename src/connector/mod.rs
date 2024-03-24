use std::{
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
};

use crate::{
    common::{error::Error, network::IPCollector},
    tunnels::{
        ring_tunnel::RingTunnelConnector, tcp_tunnel::TcpTunnelConnector,
        udp_tunnel::UdpTunnelConnector, TunnelConnector,
    },
};

pub mod direct;
pub mod manual;
pub mod udp_hole_punch;

async fn set_bind_addr_for_peer_connector(
    connector: &mut impl TunnelConnector,
    is_ipv4: bool,
    ip_collector: &Arc<IPCollector>,
) {
    let ips = ip_collector.collect_ip_addrs().await;
    if is_ipv4 {
        let mut bind_addrs = vec![];
        for ipv4 in ips.interface_ipv4s {
            let socket_addr = SocketAddrV4::new(ipv4.parse().unwrap(), 0).into();
            bind_addrs.push(socket_addr);
        }
        connector.set_bind_addrs(bind_addrs);
    } else {
        let mut bind_addrs = vec![];
        for ipv6 in ips.interface_ipv6s {
            let socket_addr = SocketAddrV6::new(ipv6.parse().unwrap(), 0, 0, 0).into();
            bind_addrs.push(socket_addr);
        }
        connector.set_bind_addrs(bind_addrs);
    }
    let _ = connector;
}

pub async fn create_connector_by_url(
    url: &str,
    ip_collector: Arc<IPCollector>,
) -> Result<Box<dyn TunnelConnector + Send + Sync + 'static>, Error> {
    let url = url::Url::parse(url).map_err(|_| Error::InvalidUrl(url.to_owned()))?;
    match url.scheme() {
        "tcp" => {
            let dst_addr =
                crate::tunnels::check_scheme_and_get_socket_addr::<SocketAddr>(&url, "tcp")?;
            let mut connector = TcpTunnelConnector::new(url);
            set_bind_addr_for_peer_connector(&mut connector, dst_addr.is_ipv4(), &ip_collector)
                .await;
            return Ok(Box::new(connector));
        }
        "udp" => {
            let dst_addr =
                crate::tunnels::check_scheme_and_get_socket_addr::<SocketAddr>(&url, "udp")?;
            let mut connector = UdpTunnelConnector::new(url);
            set_bind_addr_for_peer_connector(&mut connector, dst_addr.is_ipv4(), &ip_collector)
                .await;
            return Ok(Box::new(connector));
        }
        "ring" => {
            crate::tunnels::check_scheme_and_get_socket_addr::<uuid::Uuid>(&url, "ring")?;
            let connector = RingTunnelConnector::new(url);
            return Ok(Box::new(connector));
        }
        _ => {
            return Err(Error::InvalidUrl(url.into()));
        }
    }
}
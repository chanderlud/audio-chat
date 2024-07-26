use libp2p::swarm::NetworkBehaviour;
use libp2p::{autonat, dcutr, identify, ping, relay};

#[derive(NetworkBehaviour)]
pub(crate) struct Behaviour {
    pub(crate) relay_client: relay::client::Behaviour,
    pub(crate) ping: ping::Behaviour,
    pub(crate) identify: identify::Behaviour,
    pub(crate) dcutr: dcutr::Behaviour,
    pub(crate) stream: libp2p_stream::Behaviour,
    pub(crate) auto_nat: autonat::Behaviour,
}

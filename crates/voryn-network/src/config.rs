//! Network configuration constants and defaults.

/// Default libp2p listen port (0 = random OS-assigned port).
pub const DEFAULT_LISTEN_PORT: u16 = 0;

/// Bootstrap node listen port (fixed for server deployment).
pub const BOOTSTRAP_LISTEN_PORT: u16 = 4001;

/// Maximum number of concurrent peer connections.
pub const MAX_CONNECTIONS: usize = 50;

/// Kademlia replication factor.
pub const KAD_REPLICATION_FACTOR: usize = 20;

/// Interval between DHT refresh queries (seconds).
pub const DHT_REFRESH_INTERVAL_SECS: u64 = 300;

/// Production bootstrap peers.
/// These are hardcoded into the app and used for initial DHT discovery.
/// Replace the placeholder PeerIds with actual values after deploying nodes.
#[cfg(not(any(feature = "dev", feature = "staging")))]
pub const BOOTSTRAP_PEERS: &[&str] = &[
    // TODO: Replace <PEER_ID> with actual PeerId after bootstrap node deployment
    // "/dns4/boot1.voryn.bitstack.website/tcp/4001/p2p/<PEER_ID>",
];

/// Staging bootstrap peers.
#[cfg(feature = "staging")]
pub const BOOTSTRAP_PEERS: &[&str] = &[
    // "/dns4/boot-staging.voryn.bitstack.website/tcp/4001/p2p/12D3KooW...",
];

/// Development — no bootstrap peers, use mDNS only.
#[cfg(feature = "dev")]
pub const BOOTSTRAP_PEERS: &[&str] = &[];

/// Update server URL for checking app versions.
#[cfg(not(any(feature = "dev", feature = "staging")))]
pub const UPDATE_SERVER_URL: &str = "https://updates.voryn.bitstack.website/version.json";

#[cfg(feature = "staging")]
pub const UPDATE_SERVER_URL: &str = "https://staging.updates.voryn.bitstack.website/version.json";

#[cfg(feature = "dev")]
pub const UPDATE_SERVER_URL: &str = "";

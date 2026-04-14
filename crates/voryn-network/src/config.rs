//! Network configuration constants and defaults.

/// Default QUIC listen port (0 = random OS-assigned port).
pub const DEFAULT_LISTEN_PORT: u16 = 0;

/// Maximum number of concurrent peer connections.
pub const MAX_CONNECTIONS: usize = 50;

/// Kademlia replication factor.
pub const KAD_REPLICATION_FACTOR: usize = 20;

/// Interval between DHT refresh queries (seconds).
pub const DHT_REFRESH_INTERVAL_SECS: u64 = 300;

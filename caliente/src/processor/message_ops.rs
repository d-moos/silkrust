/// Known Message Operations

pub(crate) mod net_engine {
    pub(crate) const HANDSHAKE: usize = 0;
}

pub(crate) mod framework {
    pub(crate) const MODULE_IDENTIFICATION: usize = 1;
    pub(crate) const KEEP_ALIVE: usize = 2;
    pub(crate) const SHARD_LIST: usize = 257;
}
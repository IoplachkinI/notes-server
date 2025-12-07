use rand::{Rng, rng};

/// Lightweight snapshot of instance state for strategy selection
#[derive(Debug, Clone, Copy)]
pub struct InstanceSnapshot {
    pub con_count: u32,
    pub is_alive: bool,
}

pub trait BalancingStrategy: Send + Sync {
    /// `select_instance` receives a slice of instance snapshots that are currently alive
    /// (a snapshot of the system) and outputs a selected index
    /// Returns the index of the selected instance
    fn select_instance(&mut self, snapshots: &[InstanceSnapshot]) -> usize;
}

/////////////////////////////////////////////////////////////////////

pub struct RoundRobin {
    idx_to_pick: usize,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self { idx_to_pick: 0 }
    }
}

impl BalancingStrategy for RoundRobin {
    fn select_instance(&mut self, snapshots: &[InstanceSnapshot]) -> usize {
        let result = self.idx_to_pick;

        self.idx_to_pick = (self.idx_to_pick + 1) % snapshots.len();

        result
    }
}

/////////////////////////////////////////////////////////////////////

pub struct Random {}

impl Random {
    pub fn new() -> Self {
        Self {}
    }
}

impl BalancingStrategy for Random {
    fn select_instance(&mut self, snapshots: &[InstanceSnapshot]) -> usize {
        let mut rng = rng();

        rng.random_range(0..snapshots.len())
    }
}

/////////////////////////////////////////////////////////////////////

pub struct LeastConnections {}

impl LeastConnections {
    pub fn new() -> Self {
        Self {}
    }
}

impl BalancingStrategy for LeastConnections {
    fn select_instance(&mut self, snapshots: &[InstanceSnapshot]) -> usize {
        let mut least_connections: u32 = u32::MAX;
        let mut idx: usize = 0;

        for (i, snapshot) in snapshots.iter().enumerate() {
            if snapshot.is_alive && snapshot.con_count < least_connections {
                least_connections = snapshot.con_count;
                idx = i;
            }
        }

        idx
    }
}

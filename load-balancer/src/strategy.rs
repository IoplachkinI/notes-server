use std::sync::atomic::Ordering;

use rand::{Rng, rng};

use crate::instance::Instance;

pub trait BalancingStrategy: Send + Sync {
    /// `select_instance` recieves a slice of Instances that are currently alive
    /// (a snapshot of the system) and outputs a selected one
    /// Returns `None` if the slice is empty
    fn select_instance(&mut self, instances: &[Instance]) -> usize;
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
    fn select_instance(&mut self, instances: &[Instance]) -> usize {
        let result = self.idx_to_pick;

        self.idx_to_pick = (self.idx_to_pick + 1) % instances.len();

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
    fn select_instance(&mut self, instances: &[Instance]) -> usize {
        let mut rng = rng();

        rng.random_range(0..instances.len())
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
    fn select_instance(&mut self, instances: &[Instance]) -> usize {
        let mut least_connections: u32 = u32::MAX;
        let mut idx: usize = 0;

        for (i, instance) in instances.iter().enumerate() {
            if instance.is_alive() && instance.con_count.load(Ordering::Relaxed) < least_connections
            {
                least_connections = instance.con_count.load(Ordering::Relaxed);
                idx = i;
            }
        }

        idx
    }
}

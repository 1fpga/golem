use std::collections::VecDeque;
use std::time::Instant;

pub struct FpsCounter {
    ticks: VecDeque<Instant>,
}

impl Default for FpsCounter {
    fn default() -> Self {
        todo!()
    }
}

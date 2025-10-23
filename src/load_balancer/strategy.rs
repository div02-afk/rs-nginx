use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Context {
    pub size: usize,
    pub weights: Vec<u8>,
}

pub trait Strategy: Send + Sync {
    fn get_next_server(&mut self, ctx: &Context) -> usize;
}

#[derive(Debug)]
pub struct RoundRobin {
    pub current: usize,
}

#[derive(Debug)]
pub struct Random {}

#[derive(Debug)]
pub struct WeightedRoundRobin {
    pub current: usize,
    pub current_count: u8,
}
impl Strategy for RoundRobin {
    fn get_next_server(&mut self, ctx: &Context) -> usize {
        (self.current + 1) % ctx.size
    }
}

impl Strategy for Random {
    fn get_next_server(&mut self, ctx: &Context) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();

        println!("now {}", now);
        (now % TryInto::<u128>::try_into(ctx.size).unwrap())
            .try_into()
            .unwrap()
    }
}

impl Strategy for WeightedRoundRobin {
    fn get_next_server(&mut self, ctx: &Context) -> usize {
        if ctx.weights[self.current] > self.current_count {
            self.current_count += 1;
        } else {
            self.current_count = 0;
            self.current = (self.current + 1) % ctx.size;
        }
        self.current
    }
}

//TODO: make macro
pub fn get_next_server(size: usize, current: usize) -> usize {
    (current + 1) % size
}

macro_rules! round_robin {
    (size,curr) => {
        return (curr + 1) % size;
    };
}

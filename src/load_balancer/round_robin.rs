pub fn get_next_server(size: usize, current: usize) -> usize {
    (current + 1) % size
}

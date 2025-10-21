pub fn get_next_server(size: usize, current: usize) -> usize {
    let a = (current + 1) % size;

    return a;
}

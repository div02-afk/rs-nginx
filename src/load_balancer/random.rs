use std::time::{SystemTime, UNIX_EPOCH};
pub fn get_next_server(size: usize, current: usize) -> usize {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    println!("now {}", now);
    (now % TryInto::<u128>::try_into(size).unwrap())
        .try_into()
        .unwrap()
}

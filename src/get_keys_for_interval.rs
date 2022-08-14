pub fn get_keys_for_interval(from: u64, to: u64) -> Vec<u64> {
    let mut keys = vec![];
    let mut from = from;
    while from <= to {
        keys.push(from);
        from += 86400;
    }

    keys.push(from);

    keys
}

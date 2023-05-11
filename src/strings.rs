pub fn get_start_index(s: &str, start: &str) -> usize {
    match start.parse::<usize>() {
        Ok(number) => number,
        Err(_) => s.find(start).unwrap_or(0),
    }
}

pub fn get_end_index(s: &str, end: &str) -> usize {
    match end.parse::<usize>() {
        Ok(number) => number,
        Err(_) => s.find(end).unwrap_or(s.len()),
    }
}

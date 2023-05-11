pub fn get_start_index(s: &str, start: &str) -> usize {
    match start.parse::<usize>() {
        Ok(number) => number,
        Err(_) => {
            if start.is_empty() {
                0
            } else {
                s.find(start).unwrap_or(0)
            }
        }
    }
}

pub fn get_end_index(s: &str, end: &str) -> usize {
    match end.parse::<usize>() {
        Ok(number) => number,
        Err(_) => {
            if end.is_empty() {
                s.len()
            } else {
                s.find(end).unwrap_or(s.len())
            }
        }
    }
}

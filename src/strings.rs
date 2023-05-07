pub fn get_start_index(s: &str, start: &str) -> usize {
    let start = match start.parse::<usize>() {
        Ok(number) => number,
        Err(_) => {
            s.find(start).unwrap_or_else(|| 0)
        }
    };
    start
}

pub fn get_end_index(s: &str, end: &str) -> usize {
    let end = match end.parse::<usize>() {
        Ok(number) => number,
        Err(_) => {
            s.find(end).unwrap_or_else(|| s.len())
        }
    };
    end
}

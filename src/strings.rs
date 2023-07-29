use crate::config::Rule;

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

pub fn substring(s: &str, rule: &Rule) -> String {
    let start = get_start_index(s, rule.args[0].as_str());
    let end = get_end_index(s, rule.args[1].as_str());
    s[start..end].to_string()
}

pub fn insert(s: &str, rule: &Rule) -> String {
    let start = get_start_index(s, rule.args[0].as_str());
    let index = s.find(rule.args[1].as_str());
    let mut result = s.to_string();
    if let Some(index) = index {
        if index != start {
            result.insert_str(start, rule.args[1].as_str());
        }
    } else {
        result.insert_str(start, rule.args[1].as_str());
    }
    result
}

pub fn between(s: &str, rule: &Rule) -> String {
    if let Some(start_idx) = s.find(rule.args[0].as_str()) {
        let start_pos = start_idx + rule.args[0].len();
        if let Some(end_idx) = s[start_pos..].find(rule.args[1].as_str()) {
            let end_pos = start_pos + end_idx;
            return s[start_pos..end_pos].to_string();
        }
    }
    s.to_string()
}

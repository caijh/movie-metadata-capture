use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, path::Path};

fn md(filename: &str) -> String {
    let re = Regex::new(r"(md[a-z]{0,2}-?)(\d{2,})(-ep\d*|-\d*)*").unwrap();
    fill_and_to_string(re, filename, 4)
}

fn mmz(filename: &str) -> String {
    let re = Regex::new(r"(mmz-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn msd(filename: &str) -> String {
    let re = Regex::new(r"(msd-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn mky(filename: &str) -> String {
    let re = Regex::new(r"(mky-[a-z]{2}-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn yk(filename: &str) -> String {
    let re = Regex::new(r"(yk-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn pm(filename: &str) -> String {
    let re = Regex::new(r"(pm[a-z]?-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn fsog(filename: &str) -> String {
    let re = Regex::new(r"(fsog-?)(\d{2,})(-ep\d*)*").unwrap();
    fill_and_to_string(re, filename, 3)
}

fn fc2(filename: &str) -> String {
    let re = Regex::new(r"(?i)\bFC2-(PPV-)?\d+").unwrap();
    let m = re.captures(filename).unwrap();
    m.get(0).unwrap().as_str().to_string()
}

fn fill_and_to_string(re: Regex, filename: &str, zero_len: usize) -> String {
    let m = re.captures(filename).unwrap();
    let group_1 = m.get(1).unwrap().as_str().replace("-", "").to_uppercase();
    let group_2 = m.get(2).unwrap().as_str().to_string().pad_zeros(zero_len);
    let group_3 = m.get(3).map_or("", |x| x.as_str());
    format!("{}{}{}", group_1, group_2, group_3)
}

trait PadZeros {
    fn pad_zeros(&self, length: usize) -> String;
}

impl PadZeros for String {
    fn pad_zeros(&self, length: usize) -> String {
        let num_zeros = length - self.len();
        if num_zeros > 0 {
            "0".repeat(num_zeros) + self
        } else {
            self.to_owned()
        }
    }
}

lazy_static! {
    pub static ref G_TAKE_NUM_RULES: HashMap<&'static str, fn(&str) -> String> = {
        let mut m: HashMap<&str, fn(&str) -> String> = HashMap::new();
        m.insert("tokyo.*hot", |x: &str| {
            let re = Regex::new(r#"(cz|gedo|k|n|red-|se)\d{2,4}"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        });
        m.insert("carib", |x: &str| {
            let re = Regex::new(r#"\d{6}(-|_)\d{3}"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().replace('_', "-"))
                .unwrap_or_default()
        });
        m.insert("1pon|mura|paco", |x: &str| {
            let re = Regex::new(r#"\d{6}(-|_)\d{3}"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().replace('-', "_"))
                .unwrap_or_default()
        });
        m.insert("10mu", |x: &str| {
            let re = Regex::new(r#"\d{6}(-|_)\d{2}"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().replace('-', "_"))
                .unwrap_or_default()
        });
        m.insert("x-art", |x: &str| {
            let re = Regex::new(r#"x-art\.\d{2}\.\d{2}\.\d{2}"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        });
        m.insert("xxx-av", |x: &str| {
            let re = Regex::new(r#"xxx-av[^\d]*(\d{3,5})[^\d]*"#).unwrap();
            format!(
                "xxx-av-{}",
                re.captures(x).unwrap().get(1).unwrap().as_str()
            )
        });
        m.insert("heydouga", |x: &str| {
            let re = Regex::new(r#"(\d{4})[\-_](\d{3,4})[^\d]*"#).unwrap();
            format!(
                "heydouga-{}",
                re.captures(x).unwrap().get(1).unwrap().as_str()
            )
        });
        m.insert("heyzo", |x: &str| {
            let re = Regex::new(r#"heyzo[^\d]*(\d{4})"#).unwrap();
            format!("HEYZO-{}", re.captures(x).unwrap().get(1).unwrap().as_str())
        });
        m.insert("mdbk", |x: &str| {
            let re = Regex::new(r#"mdbk(-|_)(\d{4})"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().replace('-', "_"))
                .unwrap_or_default()
        });
        m.insert("mdtm", |x: &str| {
            let re = Regex::new(r#"mdtm(-|_)(\d{4})"#).unwrap();
            re.find(x)
                .map(|m| m.as_str().replace('-', "_"))
                .unwrap_or_default()
        });
        m.insert(r"\bmd[a-z]{0,2}-\d{2,}", md);
        m.insert(r"\bmmz-\d{2,}", mmz);
        m.insert(r"\bmsd-\d{2,}", msd);
        m.insert(r"\bmky-[a-z]{2,2}-\d{2,}", mky);
        m.insert(r"\byk-\d{2,3}", yk);
        m.insert(r"\bpm[a-z]?-?\d{2,}", pm);
        m.insert(r"\bfsog-?\d{2,}", fsog);
        m.insert(r"(?i)\bFC2-(PPV-)?\d+", fc2);
        m
    };
}

pub fn get_number_by_dict(filename: &str) -> Option<String> {
    for (k, v) in G_TAKE_NUM_RULES.iter() {
        if Regex::new(k).unwrap().is_match(filename) {
            return Some(v(filename));
        }
    }

    None
}

lazy_static! {
    static ref G_SPAT: Regex = Regex::new(r"(?-i)^\w+\.(cc|com|net|me|club|jp|tv|xyz|biz|wiki|info|tw|us|de)@|^22-sht\.me|^((fhd|hd|sd|1080p|720p|4K)(-|_)|(-|_)(fhd|hd|sd|1080p|720p|4K|x264|x265|uncensored|leak))").unwrap();
}

pub fn get_number(file_path: &str) -> Option<String> {
    let base_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    // Try to extract number from filename using a dictionary lookup
    if let Some(number) = get_number_by_dict(base_name) {
        return Some(number);
    }

    // Try to extract number from filename based on common patterns
    if base_name.contains("字幕组")
        || base_name.to_uppercase().contains("SUB")
        || Regex::new(r"[\u{30a0}-\u{30ff}]")
            .unwrap()
            .is_match(base_name)
    {
        let mut file_name = G_SPAT.replace_all(base_name, "").to_string();
        file_name = Regex::new(r"\[.*?]")
            .unwrap()
            .replace_all(&file_name, "")
            .to_string();
        file_name = file_name.replace(".chs", "").replace(".cht", "");
        let file_number = Regex::new(r"(.+?)\.")
            .unwrap()
            .captures(&file_name)
            .map(|c| c[1].to_string());
        return file_number;
    } else if base_name.contains('-') || base_name.contains('_') {
        let file_name = G_SPAT.replace_all(base_name, "").to_string();
        let mut file_number = Regex::new(r"\w+([-_])\d+")
            .unwrap()
            .captures(&file_name)
            .or_else(|| Regex::new(r"\w+").unwrap().captures(&file_name))
            .map(|c| c[0].to_string())?;
        file_number = Regex::new(r"([-_])c$")
            .unwrap()
            .replace(&file_number, "")
            .to_string();
        if Regex::new(r"\d+ch$").unwrap().is_match(&file_number) {
            file_number = file_number[..file_number.len() - 2].to_string();
        }
        return Some(file_number.to_uppercase());
    } else {
        // Try to extract number from filename based on FANZA CID
        if let Some(number) = Regex::new(r"[a-zA-Z]+\.\d{2}\.\d{2}\.\d{2}")
            .unwrap()
            .find(base_name)
            .map(|m| m.as_str().to_string())
        {
            return Some(number);
        }

        // Try to extract number from filename using regex
        let re_pattern = Regex::new(r#"([^<>/\\|:"*?]+)\.\w+$"#).unwrap();
        let file_name = re_pattern.captures(base_name).map(|c| c[1].to_string())?;
        return Regex::new(r"(.+?)\.")
            .unwrap()
            .captures(&file_name)
            .map(|c| c[1].to_string());
    }
}

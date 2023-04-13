
pub fn to_float(token_value: &String) -> Option<f64> {

    let mut upper_part: f64 = 0.0;
    let mut lower_part: f64 = 0.0;
    let mut chars = token_value.chars();

    while let Some(c) = chars.next() {
        if let Some(val) = c.to_digit(10) {
            upper_part = upper_part * 10.0 + val as f64;
        } else if c == '.' {
            break;
        } else {
            return None;
        }
    }

    for c in chars.rev() {
        if let Some(val) = c.to_digit(10) {
            lower_part = lower_part / 10.0 + val as f64;
        } else {
            return None;
        }
    }

    lower_part /= 10.0;

    return Some(upper_part + lower_part);
}

pub fn to_int(token_value: &String) -> Option<i64> {
    let mut result: i64 = 0;
    for c in token_value.chars() {
        if let Some(val) = c.to_digit(10) {
            result = result * 10 + val as i64;
        } else {
            return None;
        }
    }

    return Some(result);
}


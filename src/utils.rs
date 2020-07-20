pub fn round_to(v: f64, len: u32) -> f64 {
    (v * 10i32.pow(len) as f64).floor() / 10i32.pow(len) as f64
}

pub fn round_same(a: f64, b: f64) -> f64 {
    let s = a.to_string();
    let v: Vec<&str> = s.split(".").collect();
    if v.len() == 1 {
        return b.floor();
    }
    let len = v[1].len();
    (b * 10i32.pow(len as u32) as f64).floor() / 10i32.pow(len as u32) as f64
}

#[cfg(test)]
mod test {
    use super::*;

    //#[test]
    fn test_round_same() {
        let a = 1.23;
        let b = 1.4563;
        let ret = round_same(a, b);
        println!("ret: {:?}", ret);
    }
}

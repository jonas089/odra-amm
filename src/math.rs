use odra::types::{Balance};

pub fn _sqrt(y: Balance) -> Balance {
    if y == Balance::from(0) {
        return Balance::from(0);
    }
    let mut z: Balance = y / Balance::from(2) + Balance::from(1);
    let mut x: Balance = y;
    while x > z {
        x = z;
        z = (y / x + x) / Balance::from(2);
    }
    return z;
}

pub fn _min(x: Balance, y: Balance) -> Balance {
    if x < y {
        return x;
    } else {
        return y;
    }
}
use odra::types::U256;

pub fn _sqrt(y: U256) -> U256 {
    if y == U256::from(0) {
        return U256::from(0);
    }
    let mut z: U256 = y / U256::from(2) + U256::from(1);
    let mut x: U256 = y;
    while x > z {
        x = z;
        z = (y / x + x) / U256::from(2);
    }
    return z;
}

pub fn _min(x: U256, y: U256) -> U256 {
    if x < y {
        return x;
    } else {
        return y;
    }
}
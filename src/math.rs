use odra::types::{U256};

pub fn _sqrt(y: U256) -> U256 {
    if y == U256::from(0) {
        return U256::from(0);
    }
    let mut z: U256 = (y >> 1) + U256::from(1); // Initialize z to y / 2 + 1
    let mut x: U256 = y; // Initialize x to y
    while x > z {
        x = z; // Use binary search to update x
        z = (y / x + x) >> 1; // Equivalent to (y / x + x) / 2, but more efficient
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
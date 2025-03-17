pub fn gcd(a: u8, b: u8) -> u8 {
    let mut a = a;
    let mut b = b;

    while b > 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }

    a
}

pub fn lcm(numbers: &[u8]) -> u8 {
    let mut res = 1;

    for x in numbers.iter() {
        if *x == 0 {
            continue;
        }

        let tmp = gcd(res, *x);
        res = res / tmp * x;
    }

    res
}

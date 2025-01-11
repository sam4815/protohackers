pub trait PrimeCheck {
    fn is_prime(&self) -> bool;
}

impl PrimeCheck for i64 {
    fn is_prime(&self) -> bool {
        if *self <= 1 {
            return false;
        }

        let mut range = 2..=((*self as f64).sqrt() as i64);

        range.all(|n| *self % n != 0)
    }
}

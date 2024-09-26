use std::ops::Add;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
struct Nimber(usize);

impl Add for Nimber {
    type Output = Nimber;

    fn add(self, rhs: Self) -> Self::Output {
        // bitxor is the way nimbers are added.
        #[allow(clippy::suspicious_arithmetic_impl)]
        Nimber(self.0 ^ rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::Nimber;

    #[test]
    fn add() {
        assert_eq!(Nimber(2) + Nimber(2), Nimber(0));
    }
}

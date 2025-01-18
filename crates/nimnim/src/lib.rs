use std::{collections::HashSet, ops::Add};

/// A nimber is the size of a heap in a single-stack nim game.
///
/// Nim is crucial for loop-free* impartial combinatorial game theory analysis.
///
/// *This structure does not define utilities for loopy nimbers.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Nimber(pub usize);

impl Add for Nimber {
    type Output = Nimber;

    fn add(self, rhs: Self) -> Self::Output {
        // bitxor is the way nimbers are added.
        #[allow(clippy::suspicious_arithmetic_impl)]
        Nimber(self.0 ^ rhs.0)
    }
}

/// Returns Some(minimum excluded value of `list`), or `None` iff `list.is_empty()`
pub fn mex(list: &[Nimber]) -> Option<Nimber> {
    let mut mex: Option<Nimber> = None;
    let mut set: HashSet<Nimber> = HashSet::with_capacity(list.len());

    for item in list {
        if set.insert(*item) {
            if item > &mex.unwrap_or(Nimber(0)) {
                mex = Some(*item)
            }
        }
    }

    mex
}

#[cfg(test)]
mod tests {
    use crate::Nimber;

    #[test]
    fn add() {
        assert_eq!(Nimber(2) + Nimber(2), Nimber(0));
    }
}

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

impl From<Interval> for (u64, u64) {
    fn from(val: Interval) -> Self {
        (val.start, val.end)
    }
}

impl Interval {
    pub fn new(start: u64, end: u64) -> Self {
        debug_assert!(end >= start - 1); // TODO: make sure this is actually debug-only
        debug_assert!(start > 0);
        debug_assert!(end < u64::MAX);
        Interval { start, end }
    }

    pub fn empty() -> Self {
        Self::new(1, 0)
    }

    /// Returns the maximally allowed `u64` interval. We leave a margin of 1 from
    /// both `u64` bounds (`0` and `u64::MAX`) in order to support the reduction of any
    /// legal interval to an empty one by setting `end = start - 1` or `start = end + 1`
    pub fn maximal() -> Self {
        Self::new(1, u64::MAX - 1)
    }

    pub fn size(&self) -> u64 {
        // Empty intervals are indicated by `self.end == self.start - 1`, so
        // we avoid the overflow by first adding 1
        // Note: this function will panic if `self.end < self.start - 1` due to overflow
        (self.end + 1) - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn increase(&self, offset: u64) -> Self {
        Self::new(self.start + offset, self.end + offset)
    }

    pub fn decrease(&self, offset: u64) -> Self {
        Self::new(self.start - offset, self.end - offset)
    }

    pub fn increase_start(&self, offset: u64) -> Self {
        Self::new(self.start + offset, self.end)
    }

    pub fn decrease_start(&self, offset: u64) -> Self {
        Self::new(self.start - offset, self.end)
    }

    pub fn increase_end(&self, offset: u64) -> Self {
        Self::new(self.start, self.end + offset)
    }

    pub fn decrease_end(&self, offset: u64) -> Self {
        Self::new(self.start, self.end - offset)
    }

    pub fn split_half(&self) -> (Self, Self) {
        self.split_fraction(0.5)
    }

    /// Splits this interval to two parts such that their
    /// union is equal to the original interval and the first (left) part
    /// contains the given fraction of the original interval's size.
    /// Note: if the split results in fractional parts, this method rounds
    /// the first part up and the last part down.
    fn split_fraction(&self, fraction: f32) -> (Self, Self) {
        let left_size = f32::ceil(self.size() as f32 * fraction) as u64;

        (
            Self::new(self.start, self.start + left_size - 1),
            Self::new(self.start + left_size, self.end),
        )
    }

    /// Splits this interval to exactly |sizes| parts where
    /// |part_i| = sizes[i]. This method expects sum(sizes) to be exactly
    /// equal to the interval's size.
    pub fn split_exact(&self, sizes: &[u64]) -> Vec<Self> {
        assert_eq!(
            sizes.iter().sum::<u64>(),
            self.size(),
            "sum of sizes must be equal to the interval's size"
        );
        let mut start = self.start;
        sizes
            .iter()
            .map(|size| {
                let interval = Self::new(start, start + size - 1);
                start += size;
                interval
            })
            .collect()
    }

    /// Splits this interval to |sizes| parts
    /// by the allocation rule described below. This method expects sum(sizes)
    /// to be smaller or equal to the interval's size. Every part_i is
    /// allocated at least sizes[i] capacity. The remaining budget is
    /// split by an exponentially biased rule described below.
    ///
    /// This rule follows the GHOSTDAG protocol behavior where the child
    /// with the largest subtree is expected to dominate the competition
    /// for new blocks and thus grow the most. However, we may need to
    /// add slack for non-largest subtrees in order to make CPU reindexing
    /// attacks unworthy.
    pub fn split_exponential(&self, sizes: &[u64]) -> Vec<Self> {
        let interval_size = self.size();
        let sizes_sum = sizes.iter().sum::<u64>();
        assert!(
            interval_size >= sizes_sum,
            "interval's size must be greater than or equal to sum of sizes"
        );
        assert!(sizes_sum > 0, "cannot split to 0 parts");
        if interval_size == sizes_sum {
            return self.split_exact(sizes);
        }

        //
        // Add a fractional bias to every size in the provided sizes
        //

        let mut remaining_bias = interval_size - sizes_sum;
        let total_bias = remaining_bias as f64;

        let mut biased_sizes = Vec::<u64>::with_capacity(sizes.len());
        let exp_fractions = exponential_fractions(sizes);
        for (i, fraction) in exp_fractions.iter().enumerate() {
            let bias: u64 = if i == exp_fractions.len() - 1 {
                remaining_bias
            } else {
                remaining_bias.min(f64::round(total_bias * fraction) as u64)
            };
            biased_sizes.push(sizes[i] + bias);
            remaining_bias -= bias;
        }

        self.split_exact(biased_sizes.as_slice())
    }

    pub fn contains(&self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    pub fn strictly_contains(&self, other: Self) -> bool {
        self.start <= other.start && other.end < self.end
    }
}

/// Returns a fraction for each size in sizes
/// as follows:
///   fraction[i] = 2^size[i] / sum_j(2^size[j])
/// In the code below the above equation is divided by 2^max(size)
/// to avoid exploding numbers. Note that in 1 / 2^(max(size)-size[i])
/// we divide 1 by potentially a very large number, which will
/// result in loss of float precision. This is not a problem - all
/// numbers close to 0 bear effectively the same weight.
fn exponential_fractions(sizes: &[u64]) -> Vec<f64> {
    let max_size = sizes.iter().copied().max().unwrap_or_default();

    let mut fractions = sizes
        .iter()
        .map(|s| 1f64 / 2f64.powf((max_size - s) as f64))
        .collect::<Vec<f64>>();

    let fractions_sum = fractions.iter().sum::<f64>();
    for item in &mut fractions {
        *item /= fractions_sum;
    }

    fractions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_basics() {
        let interval = Interval::new(101, 164);
        let increased = interval.increase(10);
        let decreased = increased.decrease(5);
        // println!("{}", interval.clone());

        assert_eq!(interval.start + 10, increased.start);
        assert_eq!(interval.end + 10, increased.end);

        assert_eq!(interval.start + 5, decreased.start);
        assert_eq!(interval.end + 5, decreased.end);

        assert_eq!(interval.size(), 64);
        assert_eq!(Interval::maximal().size(), u64::MAX - 1);
        assert_eq!(Interval::empty().size(), 0);

        let (empty_left, empty_right) = Interval::empty().split_half();
        assert_eq!(empty_left.size(), 0);
        assert_eq!(empty_right.size(), 0);

        assert_eq!(interval.start + 10, interval.increase_start(10).start);
        assert_eq!(interval.start - 10, interval.decrease_start(10).start);
        assert_eq!(interval.end + 10, interval.increase_end(10).end);
        assert_eq!(interval.end - 10, interval.decrease_end(10).end);

        assert_eq!(interval.end, interval.increase_start(10).end);
        assert_eq!(interval.end, interval.decrease_start(10).end);
        assert_eq!(interval.start, interval.increase_end(10).start);
        assert_eq!(interval.start, interval.decrease_end(10).start);

        // println!("{:?}", Interval::maximal());
        // println!("{:?}", Interval::maximal().split_half());
    }

    #[test]
    fn test_split_exact() {
        let sizes = vec![5u64, 10, 15, 20];
        let intervals = Interval::new(1, 50).split_exact(sizes.as_slice());
        assert_eq!(intervals.len(), sizes.len());
        for i in 0..sizes.len() {
            assert_eq!(intervals[i].size(), sizes[i])
        }
    }

    #[test]
    fn test_exponential_fractions() {
        let mut exp_fractions = exponential_fractions(vec![2, 4, 8, 16].as_slice());
        // println!("{:?}", exp_fractions);
        for i in 0..exp_fractions.len() - 1 {
            assert!(exp_fractions[i + 1] > exp_fractions[i]);
        }

        exp_fractions = exponential_fractions(vec![].as_slice());
        assert_eq!(exp_fractions.len(), 0);

        exp_fractions = exponential_fractions(vec![0, 0].as_slice());
        assert_eq!(exp_fractions.len(), 2);
        assert_eq!(0.5f64, exp_fractions[0]);
        assert_eq!(exp_fractions[0], exp_fractions[1]);
    }

    #[test]
    fn test_contains() {
        assert!(Interval::new(1, 100).contains(Interval::new(1, 100)));
        assert!(Interval::new(1, 100).contains(Interval::new(1, 99)));
        assert!(Interval::new(1, 100).contains(Interval::new(2, 100)));
        assert!(Interval::new(1, 100).contains(Interval::new(2, 99)));
        assert!(!Interval::new(1, 100).contains(Interval::new(50, 150)));
        assert!(!Interval::new(1, 100).contains(Interval::new(150, 160)));
    }

    #[test]
    fn test_split_exponential() {
        struct Test {
            interval: Interval,
            sizes: Vec<u64>,
            expected: Vec<Interval>,
        }

        let tests = [
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![100u64],
                expected: vec![Interval::new(1, 100)],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![50u64, 50],
                expected: vec![Interval::new(1, 50), Interval::new(51, 100)],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![10u64, 20, 30, 40],
                expected: vec![
                    Interval::new(1, 10),
                    Interval::new(11, 30),
                    Interval::new(31, 60),
                    Interval::new(61, 100),
                ],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![25u64, 25],
                expected: vec![Interval::new(1, 50), Interval::new(51, 100)],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![1u64, 1],
                expected: vec![Interval::new(1, 50), Interval::new(51, 100)],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![33u64, 33, 33],
                expected: vec![
                    Interval::new(1, 33),
                    Interval::new(34, 66),
                    Interval::new(67, 100),
                ],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![10u64, 15, 25],
                expected: vec![
                    Interval::new(1, 10),
                    Interval::new(11, 25),
                    Interval::new(26, 100),
                ],
            },
            Test {
                interval: Interval::new(1, 100),
                sizes: vec![25u64, 15, 10],
                expected: vec![
                    Interval::new(1, 75),
                    Interval::new(76, 90),
                    Interval::new(91, 100),
                ],
            },
            Test {
                interval: Interval::new(1, 10_000),
                sizes: vec![10u64, 10, 20],
                expected: vec![
                    Interval::new(1, 20),
                    Interval::new(21, 40),
                    Interval::new(41, 10_000),
                ],
            },
            Test {
                interval: Interval::new(1, 100_000),
                sizes: vec![31_000u64, 31_000, 30_001],
                expected: vec![
                    Interval::new(1, 35_000),
                    Interval::new(35_001, 69_999),
                    Interval::new(70_000, 100_000),
                ],
            },
        ];

        for test in &tests {
            assert_eq!(
                test.expected,
                test.interval.split_exponential(test.sizes.as_slice())
            );
        }
    }
}

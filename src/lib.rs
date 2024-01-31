#![no_std]

use core::ops::{Add, Deref, Div, Rem, Sub};

use num_traits::{One, Zero};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidValues;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MixedRadixCounter<T, const E: usize> {
    elements: [T; E],
    limits: [T; E],
}

impl<T, const E: usize> Deref for MixedRadixCounter<T, E> {
    type Target = [T; E];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<T, const E: usize> MixedRadixCounter<T, E>
where
    T: One,
    T: Add<Output = T> + Default,
    T: PartialOrd<T> + Copy,
{
    pub fn increment(&mut self) -> Option<T> {
        for i in (0..E).rev() {
            // this can't overflow since `elements[i] < limit <= T::MAX`
            let sum = self.elements[i].add(T::one());
            if sum < self.limits[i] {
                self.elements[i] = sum;
                return None;
            }
            self.elements[i] = T::default();
        }
        Some(T::one())
    }

    pub fn add(&mut self, value: T) -> Option<T>
    where
        T: Zero,
        T: Sub<T, Output = T> + Div<T, Output = T> + Rem<T, Output = T>,
    {
        let mut carry = value;
        for i in (0..E).rev() {
            let limit = self.limits[i];
            let sum = self.elements[i].add(carry); // TODO: do we need to worry about overflows here?

            if sum < limit {
                self.elements[i] = sum;
                return None;
            } else {
                let new_value = sum % limit;
                carry = sum / limit;

                self.elements[i] = new_value;

                // if this is the last element and there is still a carry, return the overflow
                if i == 0 && carry != T::zero() {
                    return Some(carry);
                }
            }
        }
        None
    }
}

impl<T, const E: usize> TryFrom<[T; E]> for MixedRadixCounter<T, E>
where
    T: Default + Copy + PartialOrd<T>,
{
    type Error = InvalidValues;

    fn try_from(value: [T; E]) -> Result<Self, Self::Error> {
        Self::try_from_limits(value)
    }
}

impl<T, const E: usize> MixedRadixCounter<T, E>
where
    T: Default + Copy + PartialOrd<T>,
{
    pub fn try_from_limits(limits: [T; E]) -> Result<Self, InvalidValues> {
        Self::try_from_limits_and_elements(limits, [T::default(); E])
    }

    pub fn try_from_limits_and_elements(
        limits: [T; E],
        elements: [T; E],
    ) -> Result<Self, InvalidValues> {
        elements
            .iter()
            .zip(limits.iter())
            .try_for_each(|(&element, &limit)| {
                if element >= limit {
                    return Err(InvalidValues);
                }
                Ok(())
            })?;
        Ok(Self { elements, limits })
    }
}

#[cfg(test)]
mod tests {
    use crate::MixedRadixCounter;

    #[test]
    fn test_increment() {
        let mut mrc = MixedRadixCounter::try_from_limits([2_u8, 4, 3]).unwrap();
        assert_eq!(*mrc, [0, 0, 0]);

        for expected_elements in [
            [0, 0, 1],
            [0, 0, 2],
            [0, 1, 0],
            [0, 1, 1],
            [0, 1, 2],
            [0, 2, 0],
            [0, 2, 1],
            [0, 2, 2],
            [0, 3, 0],
            [0, 3, 1],
            [0, 3, 2],
            [1, 0, 0],
            [1, 0, 1],
            [1, 0, 2],
            [1, 1, 0],
            [1, 1, 1],
            [1, 1, 2],
            [1, 2, 0],
            [1, 2, 1],
            [1, 2, 2],
            [1, 3, 0],
            [1, 3, 1],
            [1, 3, 2],
        ] {
            assert!(mrc.increment().is_none());
            assert_eq!(*mrc, expected_elements);
        }

        assert_eq!(Some(1), mrc.increment());
        assert_eq!(*mrc, [0, 0, 0]);
    }

    #[test]
    fn test_counter() {
        let mut mrc = MixedRadixCounter::try_from_limits([10_u8, 10, 10, 10]).unwrap();
        assert_eq!(*mrc, [0, 0, 0, 0]);
        for i in 1_usize..10_000 {
            assert!(mrc.increment().is_none());
            let num = mrc.elements.iter().fold(0, |acc, &x| acc * 10 + x as usize);
            assert_eq!(num, i);
        }
    }

    #[test]
    fn test_large_increment() {
        let mut mrc =
            MixedRadixCounter::try_from_limits([u64::MAX, 365, 24, 60, 60, 1000]).unwrap();

        for _ in 0..69_413_798 {
            mrc.increment();
        }
        assert_eq!(*mrc, [0, 0, 19, 16, 53, 798]);
    }

    #[test]
    fn test_large_add() {
        let mut mrc =
            MixedRadixCounter::try_from_limits([u64::MAX, 365, 24, 60, 60, 1000]).unwrap();

        mrc.add(69_413_798);

        assert_eq!(*mrc, [0, 0, 19, 16, 53, 798]);
    }

    #[test]
    fn test_overflow_return() {
        for (value, expected_overflow) in [(3, 1), (4, 2), (5, 2), (6, 3), (9, 4)] {
            let mut mrc = MixedRadixCounter::try_from_limits([2_u8]).unwrap();
            assert_eq!(mrc.add(value), Some(expected_overflow));
        }

        for (value, expected_overflow) in [(4, 1), (8, 2), (9, 2), (240, 60)] {
            let mut mrc = MixedRadixCounter::try_from_limits([2_u8, 2]).unwrap();
            assert_eq!(mrc.add(value), Some(expected_overflow));
        }
    }

    #[test]
    fn test_max() {
        let mut mrc = MixedRadixCounter::try_from_limits([u8::MAX, u8::MAX]).unwrap();
        mrc.add(u8::MAX);
        assert_eq!(*mrc, [1, 0]);

        mrc.increment();
        assert_eq!(*mrc, [1, 1]);
    }
}

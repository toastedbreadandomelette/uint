use crate::Split;

use super::{count_bits, ParseUintError, ThenOr};

/// An extended 32-byte (or 256-bit) unsigned integer
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct U256([u64; 4]);

impl Split for u128 {
    #[inline(always)]
    fn split(self) -> (u64, u64) {
        ((self >> 64) as u64, self as u64)
    }
}

impl core::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self
            .display_values()
            .iter()
            .rev()
            .enumerate()
            .map(|(index, c)| {
                if index == 0 {
                    format!("{}", c)
                } else {
                    format!("{:018}", c)
                }
            })
            .collect::<Vec<String>>()
            .join("");
        f.write_fmt(format_args!("{}", s))
    }
}

impl core::fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0[0] > 0 {
            f.write_fmt(format_args!(
                "{:x}{:016x}{:016x}{:016x}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            ))
        } else if self.0[1] > 0 {
            f.write_fmt(format_args!(
                "{:x}{:016x}{:016x}",
                self.0[1], self.0[2], self.0[3]
            ))
        } else if self.0[2] > 0 {
            f.write_fmt(format_args!("{:x}{:016x}", self.0[2], self.0[3]))
        } else {
            f.write_fmt(format_args!("{:x}", self.0[3]))
        }
    }
}

impl core::fmt::UpperHex for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0[0] > 0 {
            f.write_fmt(format_args!(
                "{:X}{:016X}{:016X}{:016X}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            ))
        } else if self.0[1] > 0 {
            f.write_fmt(format_args!(
                "{:X}{:016X}{:016X}",
                self.0[1], self.0[2], self.0[3]
            ))
        } else if self.0[2] > 0 {
            f.write_fmt(format_args!("{:X}{:016X}", self.0[2], self.0[3]))
        } else {
            f.write_fmt(format_args!("{:X}", self.0[3]))
        }
    }
}

impl ThenOr for bool {
    #[inline(always)]
    fn then_or<T, B, Result>(self, fn1: T, fn2: B) -> Result
    where
        T: Fn() -> Result,
        B: Fn() -> Result,
    {
        if self {
            fn1()
        } else {
            fn2()
        }
    }

    #[inline(always)]
    fn then_val<Result>(
        self,
        true_value: Result,
        false_value: Result,
    ) -> Result {
        if self {
            true_value
        } else {
            false_value
        }
    }
}

impl U256 {
    /// Maximum value for `U256`
    pub const MAX: Self = U256([0xFFFF_FFFF_FFFF_FFFF; 4]);

    /// Minimum possible value for `U256`
    pub const MIN: Self = U256([0; 4]);

    /// Minimum possible value for `U256`
    pub const ZERO: Self = U256([0; 4]);

    /// Constant Value: 1 for `U256`
    pub const ONE: Self = U256([0, 0, 0, 1]);

    /// Create new from Raw value
    #[inline(always)]
    pub const fn raw(value: [u64; 4]) -> Self {
        Self(value)
    }

    /// Create raw value of unsigned integer
    #[inline(always)]
    #[allow(unused)]
    pub(super) fn get_raw(self) -> [u64; 4] {
        self.0
    }

    #[inline(always)]
    pub const fn raw_eq(self, other: [u64; 4]) -> bool {
        self.0[0] == other[0]
            && self.0[1] == other[1]
            && self.0[2] == other[2]
            && self.0[3] == other[3]
    }

    /// Create new value from string number
    /// # Error
    /// Returns `ParseUintError` if invalid number is found
    ///
    /// ### Example
    /// ```
    /// use uint::u256::U256;
    ///
    /// let p = U256::from_string("12");
    /// assert!(p.is_ok_and(|c| c.raw_eq([0, 0, 0, 12])));
    /// let p = U256::from_string("12312");
    /// assert!(p.is_ok_and(|c| c.raw_eq([0, 0, 0, 12312])));
    /// let p = U256::from_string("12312aa");
    /// assert!(p.is_err());
    /// ```
    pub fn from_string(string: &str) -> Result<Self, ParseUintError> {
        let mut value = U256::ZERO;

        for chr in string.chars().filter(|c| *c != '_') {
            let digit = chr.to_digit(10);
            if let Some(dig) = digit {
                let (value_x_8, value_x_2) = (value << 3, value << 1);
                value = (value_x_8 + value_x_2).add_single(dig as u64);
            } else {
                return Err(ParseUintError::InvalidDigit);
            }
        }

        Ok(value)
    }

    #[inline]
    fn check_radixes(string: &str, radix: u32) -> bool {
        match radix {
            2 => string.starts_with("0b"),
            8 => string.starts_with("0o"),
            16 => string.starts_with("0x"),
            _ => false,
        }
    }

    /// Create new value from string number, returns
    /// # Error
    /// Returns ParseUintError if invalid number is found
    ///
    /// ### Example
    /// ```
    /// use uint::u256::U256;
    ///
    /// let p = U256::from_string_radix_pow_2("0x12", 16);
    /// assert!(p.is_ok_and(|x| x.raw_eq([0, 0, 0, 0x12])));
    /// let p = U256::from_string_radix_pow_2("0o12312", 8);
    /// assert!(p.is_ok_and(|x| x.raw_eq([0, 0, 0, 0o12312])));
    /// let p = U256::from_string_radix_pow_2("0b11110011", 2);
    /// assert!(p.is_ok_and(|x| x.raw_eq([0, 0, 0, 0b11110011])));
    /// ```
    pub fn from_string_radix_pow_2(
        string: &str,
        radix: u32,
    ) -> Result<Self, ParseUintError> {
        let mut value = U256::ZERO;

        let skip = if Self::check_radixes(string, radix) {
            2
        } else {
            0
        };
        let pow = match radix {
            2 => 1,
            8 => 3,
            16 => 4,
            _ => return Err(ParseUintError::InvalidRadix),
        };

        for chr in string.chars().skip(skip).filter(|c| *c != '_') {
            let digit = chr.to_digit(radix);

            if let Some(dig) = digit {
                let value_raised_pow = value << pow;
                value = value_raised_pow ^ (dig as u64);
            } else {
                return Err(ParseUintError::InvalidDigit);
            }
        }

        Ok(value)
    }

    #[inline]
    fn add_internal(self, other: Self) -> Self {
        let (mut arr, mut index) = ([0; 4], 3);
        let (mut carry, mut is_overflown) = (0, false);

        while !is_overflown {
            let value = self.0[index] as u128 + other.0[index] as u128 + carry;
            arr[index] = value as u64;
            carry = value >> 64;
            (index, is_overflown) = index.overflowing_sub(1);
        }

        Self(arr)
    }

    #[inline(always)]
    fn add_single(self, other: u64) -> Self {
        let mut p = self.0;
        p[3] = self.0[3].wrapping_add(other);
        p[2] = self.0[2].wrapping_add((p[3] < self.0[3]).then_val(1, 0));
        p[1] = self.0[1].wrapping_add((p[2] < self.0[2]).then_val(1, 0));
        p[0] = self.0[0].wrapping_add((p[1] < self.0[1]).then_val(1, 0));

        Self(p)
    }

    #[inline]
    #[allow(unused)]
    fn mul_single(self, other: u64) -> Self {
        // Multiply first half and second half
        let (first_half, second_half) = (other & 0xFFFF_FFFF, other >> 32);
        let mut answer_first_lsb = [
            self.0[0] & 0xFFFF_FFFF,
            self.0[1] & 0xFFFF_FFFF,
            self.0[2] & 0xFFFF_FFFF,
            self.0[3] & 0xFFFF_FFFF,
        ];
        let mut answer_second_lsb = [
            self.0[0] >> 32,
            self.0[1] >> 32,
            self.0[2] >> 32,
            self.0[3] >> 32,
        ];
        let mut answer_first_msb = [
            self.0[0] & 0xFFFF_FFFF,
            self.0[1] & 0xFFFF_FFFF,
            self.0[2] & 0xFFFF_FFFF,
            self.0[3] & 0xFFFF_FFFF,
        ];
        let mut answer_second_msb = [
            self.0[0] >> 32,
            self.0[1] >> 32,
            self.0[2] >> 32,
            self.0[3] >> 32,
        ];

        answer_first_lsb[0] *= first_half;
        answer_first_lsb[1] *= first_half;
        answer_first_lsb[2] *= first_half;
        answer_first_lsb[3] *= first_half;

        answer_second_lsb[0] *= first_half;
        answer_second_lsb[1] *= first_half;
        answer_second_lsb[2] *= first_half;
        answer_second_lsb[3] *= first_half;

        answer_first_msb[0] *= second_half;
        answer_first_msb[1] *= second_half;
        answer_first_msb[2] *= second_half;
        answer_first_msb[3] *= second_half;

        answer_second_msb[0] *= second_half;
        answer_second_msb[1] *= second_half;
        answer_second_msb[2] *= second_half;
        answer_second_msb[3] *= second_half;

        let mut answer =
            U256::raw(answer_first_lsb) + (U256::raw(answer_second_lsb) << 32);
        let second_answer = (U256::raw(answer_first_msb))
            + (U256::raw(answer_second_msb) << 32);
        answer += second_answer << 32;

        answer
    }

    #[inline]
    fn mul_single_other(self, other: u64) -> Self {
        // Multiply first half and second half
        let other_u128 = other as u128;
        let mut answer = [
            (self.0[0]) as u128,
            (self.0[1]) as u128,
            (self.0[2]) as u128,
            (self.0[3]) as u128,
        ];

        answer[3] = answer[3].wrapping_mul(other_u128);
        answer[2] = answer[2].wrapping_mul(other_u128);
        answer[1] = answer[1].wrapping_mul(other_u128);
        answer[0] = answer[0].wrapping_mul(other_u128);

        answer[2] = answer[2].wrapping_add(answer[3] >> 64);
        answer[1] = answer[1].wrapping_add(answer[2] >> 64);
        answer[0] = answer[0].wrapping_add(answer[1] >> 64);

        answer[3] &= 0xFFFF_FFFF_FFFF_FFFF;
        answer[2] &= 0xFFFF_FFFF_FFFF_FFFF;
        answer[1] &= 0xFFFF_FFFF_FFFF_FFFF;
        answer[0] &= 0xFFFF_FFFF_FFFF_FFFF;

        U256::raw(answer.map(|c| c as u64))
    }

    fn div_internal(self, other: Self) -> Self {
        match self.cmp(&other) {
            core::cmp::Ordering::Less => Self::ZERO,
            core::cmp::Ordering::Equal => Self::ONE,
            _ => {
                let div = other;
                let mut divisor = other;
                let leading_zeros = divisor.leading_zeros();
                divisor <<= leading_zeros - self.leading_zeros();

                let mut value = self;
                let mut quotient = Self::MIN;

                while value >= div {
                    while value < divisor {
                        divisor >>= 1;
                        quotient <<= 1;
                    }

                    value -= divisor;
                    quotient = quotient.add_single(1);
                }
                let rem_offset = div.leading_zeros() - divisor.leading_zeros();

                quotient << rem_offset
            }
        }
    }

    fn rem_internal(self, other: Self) -> Self {
        match self.cmp(&other) {
            core::cmp::Ordering::Less => self,
            core::cmp::Ordering::Equal => Self::ZERO,
            _ => {
                let div = other;
                let mut divisor = other;
                let leading_zeros = divisor.leading_zeros();
                divisor <<= leading_zeros - self.leading_zeros();

                let mut value = self;

                while value >= div {
                    while value < divisor {
                        divisor >>= 1;
                    }

                    value -= divisor;
                }

                value
            }
        }
    }

    fn div_rem_internal(self, other: Self) -> (Self, Self) {
        match self.cmp(&other) {
            core::cmp::Ordering::Less => (Self::ZERO, self),
            core::cmp::Ordering::Equal => (Self::ONE, Self::ZERO),
            _ => {
                let div = other;
                let mut divisor = other;
                let leading_zeros = divisor.leading_zeros();
                divisor <<= leading_zeros - self.leading_zeros();

                let mut value = self;
                let mut quotient = Self::ZERO;

                while value >= div {
                    while value < divisor {
                        divisor >>= 1;
                        quotient <<= 1;
                    }

                    value -= divisor;
                    quotient = quotient.add_single(1);
                }
                let rem_offset = div.leading_zeros() - divisor.leading_zeros();

                (quotient << rem_offset, value)
            }
        }
    }

    fn display_values(self) -> Vec<u64> {
        let divisor = Self::from(1_000_000_000_000_000_000_u64);
        let mut value = self;
        let mut values = vec![];

        while !value.is_zero() {
            let (q, rem) = value.div_rem_internal(divisor);
            value = q;
            values.push(rem.0[3]);
        }

        values
    }

    pub fn div_single(self, divisor: u64) -> Self {
        if self.0[0] == 0 && self.0[1] == 0 {
            if self.0[2] == 0 {
                Self::from(self.0[3] / divisor)
            } else {
                let value_128 = u128::from(self) / u128::from(divisor);
                Self::from(value_128)
            }
        } else {
            let div = Self::from(divisor);
            let mut div_256 = Self::from(divisor);
            let leading_zeros = div.leading_zeros();
            div_256 <<= self.leading_zeros() - leading_zeros;

            let mut value = self;
            let mut quotient = U256::ZERO;

            while value >= div {
                while value < div_256 {
                    div_256 >>= 1;
                    quotient <<= 1;
                }

                value -= div_256;
                quotient = quotient.add_single(1);
            }
            let rem_offset = div.leading_zeros() - divisor.leading_zeros();

            quotient << rem_offset
        }
    }

    pub fn rem_single(self, divisor: u64) -> Self {
        if self.0[0] == 0 && self.0[1] == 0 {
            if self.0[2] == 0 {
                Self::from(self.0[3] % divisor)
            } else {
                let value_128 = u128::from(self) % u128::from(divisor);
                Self::from(value_128)
            }
        } else {
            let div = Self::from(divisor);
            let mut div_256 = Self::from(divisor);
            let leading_zeros = div.leading_zeros();
            div_256 <<= self.leading_zeros() - leading_zeros;
            let mut value = self;

            while value >= div {
                while value < div_256 {
                    div_256 >>= 1;
                }

                value -= div_256;
            }

            value
        }
    }

    #[inline(always)]
    fn madd_split(a: u64, b: u64, c: u64) -> (u64, u64) {
        (u128::from(a) * u128::from(b) + u128::from(c)).split()
    }

    #[inline]
    fn mul_internal(self, other: U256) -> Self {
        let mut answer: [u64; 4] = [0; 4];

        let (hcarry, lcarry) = Self::madd_split(self.0[3], other.0[3], 0);
        answer[3] = answer[3].wrapping_add(lcarry);
        let (hcarry, lcarry) = Self::madd_split(self.0[2], other.0[3], hcarry);
        answer[2] = answer[2].wrapping_add(lcarry);
        let (hcarry, lcarry) = Self::madd_split(self.0[1], other.0[3], hcarry);
        answer[1] = answer[1].wrapping_add(lcarry);
        let (_, lcarry) = Self::madd_split(self.0[0], other.0[3], hcarry);
        answer[0] = answer[0].wrapping_add(lcarry);

        let (hcarry, lcarry) = Self::madd_split(self.0[3], other.0[2], 0);
        answer[2] = answer[2].wrapping_add(lcarry);
        let (hcarry, lcarry) = Self::madd_split(self.0[2], other.0[2], hcarry);
        answer[1] = answer[1].wrapping_add(lcarry);
        let (_, lcarry) = Self::madd_split(self.0[1], other.0[2], hcarry);
        answer[0] = answer[0].wrapping_add(lcarry);

        let (hcarry, lcarry) = Self::madd_split(self.0[3], other.0[1], 0);
        answer[1] = answer[1].wrapping_add(lcarry);
        let (_, lcarry) = Self::madd_split(self.0[2], other.0[1], hcarry);
        answer[0] = answer[0].wrapping_add(lcarry);

        let (_, lcarry) = Self::madd_split(self.0[3], other.0[0], 0);
        answer[0] = answer[0].wrapping_add(lcarry);

        Self(answer)
    }

    /// Subtracting using 2's complement
    #[inline(always)]
    fn sub_internal(self, other: Self) -> Self {
        self + (!other + U256::ONE)
    }

    fn shift_left_internal(self, rhs: u32) -> Self {
        let (arr_offset, shft) = (rhs >> 6, rhs & 63);
        if shft > 0 {
            match arr_offset {
                0 => Self([
                    (self.0[0].wrapping_shl(shft))
                        | (self.0[1].wrapping_shr(64 - shft)),
                    (self.0[1].wrapping_shl(shft))
                        | (self.0[2].wrapping_shr(64 - shft)),
                    (self.0[2].wrapping_shl(shft))
                        | (self.0[3].wrapping_shr(64 - shft)),
                    self.0[3].wrapping_shl(shft),
                ]),
                1 => Self([
                    (self.0[1].wrapping_shl(shft))
                        | (self.0[2].wrapping_shr(64 - shft)),
                    (self.0[2].wrapping_shl(shft))
                        | (self.0[3].wrapping_shr(64 - shft)),
                    self.0[3].wrapping_shl(shft),
                    0,
                ]),
                2 => Self([
                    (self.0[2].wrapping_shl(shft))
                        | (self.0[3].wrapping_shr(64 - shft)),
                    self.0[3].wrapping_shl(shft),
                    0,
                    0,
                ]),
                3 => Self([self.0[3].wrapping_shl(shft), 0, 0, 0]),
                _ => U256::ZERO,
            }
        } else {
            match arr_offset {
                0 => self,
                1 => Self([self.0[1], self.0[2], self.0[3], 0]),
                2 => Self([self.0[2], self.0[3], 0, 0]),
                3 => Self([self.0[3], 0, 0, 0]),
                _ => U256::ZERO,
            }
        }
    }

    fn shift_right_internal(self, rhs: u32) -> Self {
        let (arr_offset, shft) = (rhs >> 6, rhs & 63);
        if shft > 0 {
            match arr_offset {
                0 => Self([
                    self.0[0].wrapping_shr(shft),
                    (self.0[1].wrapping_shr(shft))
                        | (self.0[0].wrapping_shl(64 - shft)),
                    (self.0[2].wrapping_shr(shft))
                        | (self.0[1].wrapping_shl(64 - shft)),
                    (self.0[3].wrapping_shr(shft))
                        | (self.0[2].wrapping_shl(64 - shft)),
                ]),
                1 => Self([
                    0,
                    self.0[0].wrapping_shr(shft),
                    (self.0[1].wrapping_shr(shft))
                        | (self.0[0].wrapping_shl(64 - shft)),
                    (self.0[2].wrapping_shr(shft))
                        | (self.0[1].wrapping_shl(64 - shft)),
                ]),
                2 => Self([
                    0,
                    0,
                    self.0[0].wrapping_shr(shft),
                    (self.0[1].wrapping_shr(shft))
                        | (self.0[0].wrapping_shl(64 - shft)),
                ]),
                3 => Self([0, 0, 0, self.0[0].wrapping_shr(shft)]),
                _ => U256::ZERO,
            }
        } else {
            match arr_offset {
                0 => self,
                1 => Self([0, self.0[0], self.0[1], self.0[2]]),
                2 => Self([0, 0, self.0[0], self.0[1]]),
                3 => Self([0, 0, 0, self.0[0]]),
                _ => U256::ZERO,
            }
        }
    }

    /// Convenient function to check whether this number is
    /// zero or not
    ///
    /// ### Example
    /// ```
    /// use uint::u256::U256;
    ///
    /// assert!(U256::from_string("0").is_ok_and(|c| c.is_zero()));
    /// assert!(U256::from_string("1").is_ok_and(|d| !d.is_zero()));
    /// ```
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Convenient function to check whether this number is
    /// zero or not
    ///
    /// ### Example
    /// ```
    /// use uint::u256::U256;
    ///
    /// assert!((U256::ONE << 64).trailing_zeros() == 64);
    /// assert!((U256::ONE << 192).trailing_zeros() == 192);
    /// ```
    pub const fn trailing_zeros(&self) -> u32 {
        if self.0[3] > 0 {
            self.0[3].trailing_zeros()
        } else if self.0[2] > 0 {
            64 + self.0[2].trailing_zeros()
        } else if self.0[1] > 0 {
            128 + self.0[1].trailing_zeros()
        } else if self.0[0] > 0 {
            192 + self.0[0].trailing_zeros()
        } else {
            256
        }
    }

    /// Function to check whether this number is
    /// zero or not
    ///
    /// ### Example
    /// ```
    /// use uint::u256::U256;
    ///
    /// assert!((U256::ONE << 64).leading_zeros() == 191);
    /// assert!((U256::ONE << 192).leading_zeros() == 63);
    /// ```
    pub const fn leading_zeros(&self) -> u32 {
        if self.0[0] > 0 {
            self.0[0].leading_zeros()
        } else if self.0[1] > 0 {
            64 + self.0[1].leading_zeros()
        } else if self.0[2] > 0 {
            128 + self.0[2].leading_zeros()
        } else if self.0[3] > 0 {
            192 + self.0[3].leading_zeros()
        } else {
            256
        }
    }

    #[inline(always)]
    pub const fn bits(self) -> u64 {
        count_bits(self.0[0])
            + count_bits(self.0[1])
            + count_bits(self.0[2])
            + count_bits(self.0[3])
    }
}

impl core::ops::Not for U256 {
    type Output = U256;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }
}

impl core::cmp::Ord for U256 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.0[0] != other.0[0] {
            return self.0[0].cmp(&other.0[0]);
        }
        if self.0[1] != other.0[1] {
            return self.0[1].cmp(&other.0[1]);
        }
        if self.0[2] != other.0[2] {
            return self.0[2].cmp(&other.0[2]);
        }

        self.0[3].cmp(&other.0[3])
    }
}

impl core::cmp::PartialOrd for U256 {
    #[inline]
    fn ge(&self, other: &Self) -> bool {
        if self.0[0] != other.0[0] {
            return self.0[0].gt(&other.0[0]);
        }
        if self.0[1] != other.0[1] {
            return self.0[1].gt(&other.0[1]);
        }
        if self.0[2] != other.0[2] {
            return self.0[2].gt(&other.0[2]);
        }

        self.0[3].ge(&other.0[3])
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        if self.0[0] != other.0[0] {
            return self.0[0].lt(&other.0[0]);
        }
        if self.0[1] != other.0[1] {
            return self.0[1].lt(&other.0[1]);
        }
        if self.0[2] != other.0[2] {
            return self.0[2].lt(&other.0[2]);
        }

        self.0[3].le(&other.0[3])
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        if self.0[0] != other.0[0] {
            return self.0[0].gt(&other.0[0]);
        }
        if self.0[1] != other.0[1] {
            return self.0[1].gt(&other.0[1]);
        }
        if self.0[2] != other.0[2] {
            return self.0[2].gt(&other.0[2]);
        }

        self.0[3].gt(&other.0[3])
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        if self.0[0] != other.0[0] {
            return self.0[0].lt(&other.0[0]);
        }
        if self.0[1] != other.0[1] {
            return self.0[1].lt(&other.0[1]);
        }
        if self.0[2] != other.0[2] {
            return self.0[2].lt(&other.0[2]);
        }

        self.0[3].lt(&other.0[3])
    }

    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::str::FromStr for U256 {
    type Err = ParseUintError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            Self::from_string_radix_pow_2(s, 16)
        } else if s.starts_with("0b") {
            Self::from_string_radix_pow_2(s, 2)
        } else if s.starts_with("0o") {
            Self::from_string_radix_pow_2(s, 8)
        } else {
            Self::from_string(s)
        }
    }
}

impl From<u128> for U256 {
    #[inline]
    fn from(value: u128) -> Self {
        Self([0, 0, (value >> 64) as u64, value as u64])
    }
}

impl From<u64> for U256 {
    #[inline]
    fn from(value: u64) -> Self {
        Self([0, 0, 0, value])
    }
}

impl From<u32> for U256 {
    #[inline]
    fn from(value: u32) -> Self {
        Self([0, 0, 0, value as u64])
    }
}

impl From<u16> for U256 {
    #[inline]
    fn from(value: u16) -> Self {
        Self([0, 0, 0, value as u64])
    }
}

impl From<u8> for U256 {
    #[inline]
    fn from(value: u8) -> Self {
        Self([0, 0, 0, value as u64])
    }
}

impl From<U256> for u128 {
    #[inline]
    fn from(value: U256) -> Self {
        ((value.0[2] as u128) << 64) | (value.0[3] as u128)
    }
}

impl From<U256> for u64 {
    #[inline]
    fn from(value: U256) -> u64 {
        value.0[3]
    }
}

impl From<U256> for u32 {
    #[inline]
    fn from(value: U256) -> u32 {
        value.0[3] as u32
    }
}

impl From<U256> for u16 {
    #[inline]
    fn from(value: U256) -> u16 {
        value.0[3] as u16
    }
}

impl From<U256> for u8 {
    #[inline]
    fn from(value: U256) -> u8 {
        value.0[3] as u8
    }
}

impl core::ops::Add<U256> for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, rhs: U256) -> Self::Output {
        self.add_internal(rhs)
    }
}

impl core::ops::Add<u64> for U256 {
    type Output = U256;

    #[inline(always)]
    fn add(self, rhs: u64) -> Self::Output {
        self.add_single(rhs)
    }
}

impl core::ops::AddAssign<U256> for U256 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: U256) {
        *self = self.add_internal(rhs)
    }
}

impl core::ops::Sub<U256> for U256 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: U256) -> Self::Output {
        self.sub_internal(rhs)
    }
}

impl core::ops::SubAssign<U256> for U256 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: U256) {
        *self = self.sub_internal(rhs)
    }
}

impl core::ops::Mul<U256> for U256 {
    type Output = U256;
    #[inline(always)]
    fn mul(self, other: U256) -> Self::Output {
        self.mul_internal(other)
    }
}

impl core::ops::MulAssign<U256> for U256 {
    #[inline(always)]
    fn mul_assign(&mut self, other: U256) {
        *self = self.mul_internal(other)
    }
}

impl core::ops::Mul<u64> for U256 {
    type Output = U256;
    #[inline(always)]
    fn mul(self, rhs: u64) -> Self::Output {
        self.mul_single_other(rhs)
    }
}

impl core::ops::MulAssign<u64> for U256 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: u64) {
        *self = self.mul_single(rhs)
    }
}

impl core::ops::Div<U256> for U256 {
    type Output = U256;
    #[inline(always)]
    fn div(self, other: U256) -> Self::Output {
        self.div_internal(other)
    }
}

impl core::ops::DivAssign<U256> for U256 {
    #[inline(always)]
    fn div_assign(&mut self, other: U256) {
        *self = self.div_internal(other)
    }
}

impl core::ops::Div<u64> for U256 {
    type Output = U256;
    #[inline(always)]
    fn div(self, rhs: u64) -> Self::Output {
        self.div_single(rhs)
    }
}

impl core::ops::DivAssign<u64> for U256 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: u64) {
        *self = self.div_single(rhs)
    }
}

impl core::ops::Rem<U256> for U256 {
    type Output = U256;
    #[inline(always)]
    fn rem(self, other: U256) -> Self::Output {
        self.rem_internal(other)
    }
}

impl core::ops::RemAssign<U256> for U256 {
    #[inline(always)]
    fn rem_assign(&mut self, other: U256) {
        *self = self.rem_internal(other)
    }
}

impl core::ops::Rem<u64> for U256 {
    type Output = U256;
    #[inline(always)]
    fn rem(self, rhs: u64) -> Self::Output {
        self.rem_single(rhs)
    }
}

impl core::ops::RemAssign<u64> for U256 {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: u64) {
        *self = self.rem_single(rhs)
    }
}

impl core::ops::Shl<u32> for U256 {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: u32) -> Self::Output {
        self.shift_left_internal(rhs)
    }
}

impl core::ops::ShlAssign<u32> for U256 {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u32) {
        *self = self.shift_left_internal(rhs)
    }
}

impl core::ops::Shr<u32> for U256 {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: u32) -> Self::Output {
        self.shift_right_internal(rhs)
    }
}

impl core::ops::ShrAssign<u32> for U256 {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u32) {
        *self = self.shift_right_internal(rhs)
    }
}

impl core::ops::BitAnd<U256> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: U256) -> Self::Output {
        Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl core::ops::BitAnd<u64> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self::Output {
        Self([self.0[0], self.0[1], self.0[2], self.0[3] & rhs])
    }
}

impl core::ops::BitAndAssign<U256> for U256 {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: U256) {
        *self = Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl core::ops::BitAndAssign<u64> for U256 {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: u64) {
        *self = Self([self.0[0], self.0[1], self.0[2], self.0[3] & rhs])
    }
}

impl core::ops::BitOr<U256> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: U256) -> Self::Output {
        Self([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

impl core::ops::BitOr<u64> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: u64) -> Self::Output {
        Self([self.0[0], self.0[1], self.0[2], self.0[3] | rhs])
    }
}

impl core::ops::BitOrAssign<U256> for U256 {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: U256) {
        *self = Self([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

impl core::ops::BitOrAssign<u64> for U256 {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: u64) {
        *self = Self([self.0[0], self.0[1], self.0[2], self.0[3] | rhs])
    }
}

impl core::ops::BitXor<U256> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: U256) -> Self::Output {
        Self([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
            self.0[3] ^ rhs.0[3],
        ])
    }
}

impl core::ops::BitXor<u64> for U256 {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Self([self.0[0], self.0[1], self.0[2], self.0[3] ^ rhs])
    }
}

impl core::ops::BitXorAssign<U256> for U256 {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: U256) {
        *self = Self([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
            self.0[3] ^ rhs.0[3],
        ])
    }
}

impl core::ops::BitXorAssign<u64> for U256 {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: u64) {
        *self = Self([self.0[0], self.0[1], self.0[2], self.0[3] ^ rhs])
    }
}

#[cfg(test)]
mod test {
    use super::{super::count_bits, ParseUintError, U256};

    #[test]
    fn gen_test() -> Result<(), ParseUintError> {
        let value = U256::from_string("12345678919")?;
        assert!(value.raw_eq([0, 0, 0, 12345678919]));

        // Equal to 1 << 64 = 18446744073709551616
        let value = U256::from_string("18446744073709551616")?;
        assert_eq!(value.0, ([0, 0, 1, 0]));

        // Equal to 1 << 128
        let value =
            U256::from_string("340282366920938463463374607431768211456")?;
        assert_eq!(value.0, ([0, 1, 0, 0]));

        // Equal to 115792089237316195423570985008687907852837564279074904382605163141518161494337
        let value = U256::from_string("115792089237316195423570985008687907852837564279074904382605163141518161494337")?;
        assert_eq!(
            value,
            U256([
                18446744073709551615,
                18446744073709551614,
                13451932020343611451,
                13822214165235122497
            ])
        );

        let value = U256::from_string("16983810465656793445178183341822322175883642221536626637512293983324")?;
        assert_eq!(
            value,
            U256([
                0xa1455b33,
                0x4df099df30fc28a1,
                0x69a467e9e47075a9,
                0x0f7e650eb6b7a45c
            ])
        );

        // Max value of 256-bit number
        let value = U256::from_string("115792089237316195423570985008687907853269984665640564039457584007913129639935")?;
        assert_eq!(value, U256::MAX);

        Ok(())
    }

    #[test]
    fn test_bits() {
        assert_eq!(count_bits(0b01011100), 4);
        assert_eq!(count_bits(0b01_1100_1111_0011_1110_1001), 14);
    }

    #[test]
    fn test_fmt() -> Result<(), Box<dyn std::error::Error>> {
        let num = "3912093812908391208428194902184908123982189742178629873982173391238912122312".parse::<U256>()?;

        assert_eq!(
            format!("{}", num),
            "3912093812908391208428194902184908123982189742178629873982173391238912122312"
        );
        let num = "1213232142132321321".parse::<U256>()?;

        assert_eq!(
            format!("{}", num),
            "1213232142132321321"
        );

        Ok(())
    }

    #[test]
    fn test_add() -> Result<(), ParseUintError> {
        let a = U256::from_string("1245")?;
        let b = U256::from_string("4546477")?;
        let c = a + b;
        assert_eq!(c.0, U256::from_string("4547722")?.0);

        let a = (U256::ONE << 192) - U256::ONE;
        let b = (U256::ONE << 192) - U256::ONE;
        let c = a + b;
        assert_eq!(c, (U256::ONE << 193) - (U256::ONE << 1));

        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(), ParseUintError> {
        let a = U256::from_string("12131414122")?;
        let b = U256::from_string("4546477")?;
        let c = a - b;

        assert_eq!(c, U256::from_string("12126867645")?);

        // Finite Field for secp256k1 (2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1)
        // Source: https://en.bitcoin.it/wiki/Secp256k1
        let p = U256::MAX
            ^ (U256::ONE << 32)
            ^ (U256::ONE << 9)
            ^ (U256::ONE << 8)
            ^ (U256::ONE << 7)
            ^ (U256::ONE << 6)
            ^ (U256::ONE << 4);

        assert_eq!(p, "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F".parse::<U256>()?);

        Ok(())
    }

    #[test]
    fn test_mul_single() -> Result<(), ParseUintError> {
        let a = U256::from_string("124312312142135")?;
        let b = 4546477;
        let c = a * b;

        assert_eq!(c.0, U256::from_string("565183067971037508395")?.0);

        let a = U256::from_string("29408993404948928992877151431649155974")?;
        let b = 2132132112312312321;
        let c = a * b;

        assert_eq!(
            c.0,
            U256::from_string(
                "62703859229472622214294486738735594201348857847930955654"
            )?
            .0
        );

        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(), ParseUintError> {
        let a = U256::from_string("1231242")?;
        let b = U256::from_string("42145124")?;
        let c = a * b;
        assert_eq!(c, U256::from_string("51890846764008")?);

        // Multiply 1 << 64 with 1 << 64
        let a = U256::ONE << 64;
        let b = U256::ONE << 64;
        let c = a * b;
        assert_eq!(c, U256::ONE << 128);

        let a = U256::from_string("29408993404948928992877151431649155974")?;
        let b = U256::from_string("213213211231231232131232142142421312")?;
        let c = a * b;

        assert_eq!(
            c,
            U256::from_string(
                "6270385922947262222347954536162455298520515727022267860678267425509717888"
            )?
        );

        Ok(())
    }

    #[test]
    fn test_shift_bits() -> Result<(), ParseUintError> {
        let mut a = U256::from_string("940899340494892899287232132141")?;
        a <<= 98;

        assert_eq!(
            a,
            U256::from_string(
                "298182903433174043516747888242818498075070855821604373397504"
            )?
        );

        assert_eq!(
            a >> 1,
            "149091451716587021758373944121409249037535427910802186698752"
                .parse::<U256>()?
        );

        assert_eq!(a >> 178, U256::from_string("778293")?);
        assert_eq!(a << 22, U256::from_string("1250669744601375623418469734648406597750261990855978509758644617216")?);
        assert_eq!(a << 25, U256::from_string("10005357956811004987347757877187252782002095926847828078069156937728")?);

        Ok(())
    }

    #[test]
    fn test_div() -> Result<(), ParseUintError> {
        let a = U256::from_string("29408993404948928992877151431649155974")?;
        let b = 123456566;
        let c = a / b;

        assert_eq!(c, U256::from_string("238213279032473080393935073746")?);

        let a = U256::from_string(
            "294089934049489289928723213214128471823791287151431649155974",
        )?;
        let b = U256::from_string("940899340494892899287232132141")?;
        let c = a / b;

        assert_eq!(c, U256::from_string("312562589208325179552885042139")?);

        let a = U256::from_string("3912093812908391208428194902184908123982189742178629873982173391238912")?;
        let b = U256::from_string(
            "940899340494892899287232134329048329473287439824732982141",
        )?;
        let c = a / b;

        assert_eq!(c, U256::from_string("4157823950488")?);

        let a = U256::from_string("1032105389620138683259824866402890871739720549422559896654845224087")?;
        let b = U256::from_string("51248759832749832749873129879328147")?;
        let c = a / b;

        assert_eq!(c, U256::from_string("20139129083092183902183912839021")?);

        Ok(())
    }

    #[test]
    fn test_rem() -> Result<(), ParseUintError> {
        let a = U256::from_string("29408993404948928992877151431649155974")?;
        let b = 123456566;
        let c = a % b;

        assert_eq!(c, U256::from_string("11239738")?);

        let a = U256::from_string(
            "294089934049489289928723213214128471823791287151431649155974",
        )?;
        let b = U256::from_string("940899340494892899287232132141")?;

        assert_eq!(a % b, U256::from_string("229622695252588468980047866375")?);

        let a = U256::from_string("3912093812908391208428194902184908123982189742178629873982173391238912")?;
        let b = U256::from_string(
            "940899340494892899287232134329048329473287439824732982141",
        )?;
        let c = a % b;

        assert_eq!(
            c,
            "361780925295470009804517438088754858207007846931619004104"
                .parse::<U256>()?
        );

        let a = "1032105389620138683259824866402890871739720549422559896654845224087".parse::<U256>()?;
        let b = "51248759832749832749873129879328147".parse::<U256>()?;
        let c = a % b;

        assert_eq!(c.is_zero(), true);

        Ok(())
    }
}

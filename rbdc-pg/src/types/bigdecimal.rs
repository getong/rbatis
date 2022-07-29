use std::cmp;
use std::num::TryFromIntError;
use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};
use rbdc::Error;
use crate::arguments::PgArgumentBuffer;
use crate::type_info::PgTypeInfo;
use crate::types::decode::Decode;
use crate::types::encode::{Encode, IsNull};
use crate::types::numeric::{PgNumeric, PgNumericSign};
use crate::value::{PgValue, PgValueFormat};

impl TryFrom<PgNumeric> for BigDecimal {
    type Error = Error;

    fn try_from(numeric: PgNumeric) -> Result<Self, Error> {
        let (digits, sign, weight) = match numeric {
            PgNumeric::Number {
                digits,
                sign,
                weight,
                ..
            } => (digits, sign, weight),

            PgNumeric::NotANumber => {
                return Err("BigDecimal does not support NaN values".into());
            }
        };

        if digits.is_empty() {
            // Postgres returns an empty digit array for 0 but BigInt expects at least one zero
            return Ok(0u64.into());
        }

        let sign = match sign {
            PgNumericSign::Positive => Sign::Plus,
            PgNumericSign::Negative => Sign::Minus,
        };

        // weight is 0 if the decimal point falls after the first base-10000 digit
        let scale = (digits.len() as i64 - weight as i64 - 1) * 4;

        // no optimized algorithm for base-10 so use base-100 for faster processing
        let mut cents = Vec::with_capacity(digits.len() * 2);
        for digit in &digits {
            cents.push((digit / 100) as u8);
            cents.push((digit % 100) as u8);
        }

        let bigint = BigInt::from_radix_be(sign, &cents, 100)
            .ok_or("PgNumeric contained an out-of-range digit")?;

        Ok(BigDecimal::new(bigint, scale))
    }
}

impl TryFrom<&'_ BigDecimal> for PgNumeric {
    type Error = Error;

    fn try_from(decimal: &BigDecimal) -> Result<Self, Error> {
        let base_10_to_10000 = |chunk: &[u8]| chunk.iter().fold(0i16, |a, &d| a * 10 + d as i16);

        // NOTE: this unfortunately copies the BigInt internally
        let (integer, exp) = decimal.as_bigint_and_exponent();

        // this routine is specifically optimized for base-10
        // FIXME: is there a way to iterate over the digits to avoid the Vec allocation
        let (sign, base_10) = integer.to_radix_be(10);

        // weight is positive power of 10000
        // exp is the negative power of 10
        let weight_10 = base_10.len() as i64 - exp;

        // scale is only nonzero when we have fractional digits
        // since `exp` is the _negative_ decimal exponent, it tells us
        // exactly what our scale should be
        let scale: i16 = cmp::max(0, exp).try_into().map_err(|e:TryFromIntError|Error::from(e.to_string()))?;

        // there's an implicit +1 offset in the interpretation
        let weight: i16 = if weight_10 <= 0 {
            weight_10 / 4 - 1
        } else {
            // the `-1` is a fix for an off by 1 error (4 digits should still be 0 weight)
            (weight_10 - 1) / 4
        }
        .try_into().map_err(|e:TryFromIntError|Error::from(e.to_string()))?;

        let digits_len = if base_10.len() % 4 != 0 {
            base_10.len() / 4 + 1
        } else {
            base_10.len() / 4
        };

        let offset = weight_10.rem_euclid(4) as usize;

        let mut digits = Vec::with_capacity(digits_len);

        if let Some(first) = base_10.get(..offset) {
            if !first.is_empty() {
                digits.push(base_10_to_10000(first));
            }
        } else if offset != 0 {
            digits.push(base_10_to_10000(&base_10) * 10i16.pow((offset - base_10.len()) as u32));
        }

        if let Some(rest) = base_10.get(offset..) {
            digits.extend(
                rest.chunks(4)
                    .map(|chunk| base_10_to_10000(chunk) * 10i16.pow(4 - chunk.len() as u32)),
            );
        }

        while let Some(&0) = digits.last() {
            digits.pop();
        }

        Ok(PgNumeric::Number {
            sign: match sign {
                Sign::Plus | Sign::NoSign => PgNumericSign::Positive,
                Sign::Minus => PgNumericSign::Negative,
            },
            scale,
            weight,
            digits,
        })
    }
}

/// ### Panics
/// If this `BigDecimal` cannot be represented by `PgNumeric`.
impl Encode for BigDecimal {
    fn encode(self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Error> {
            PgNumeric::try_from(&self)
                .expect("BigDecimal magnitude too great for Postgres NUMERIC type")
                .encode(buf);
            Ok(IsNull::No)
    }
}

impl Decode for BigDecimal {
    fn decode(value: PgValue) -> Result<Self, Error> {
       match value.format() {
            PgValueFormat::Binary => PgNumeric::decode(value.as_bytes()?)?.try_into(),
            PgValueFormat::Text => Ok(value.as_str()?.parse::<BigDecimal>().map_err(|e|Error::from(e.to_string()))?),
        }
    }
}

#[cfg(test)]
mod bigdecimal_to_pgnumeric {
    use super::{BigDecimal, PgNumeric, PgNumericSign};
    use std::convert::TryFrom;

    #[test]
    fn zero() {
        let zero: BigDecimal = "0".parse().unwrap();

        assert_eq!(
            PgNumeric::try_from(&zero).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 0,
                digits: vec![]
            }
        );
    }

    #[test]
    fn one() {
        let one: BigDecimal = "1".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&one).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 0,
                digits: vec![1]
            }
        );
    }

    #[test]
    fn ten() {
        let ten: BigDecimal = "10".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&ten).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 0,
                digits: vec![10]
            }
        );
    }

    #[test]
    fn one_hundred() {
        let one_hundred: BigDecimal = "100".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&one_hundred).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 0,
                digits: vec![100]
            }
        );
    }

    #[test]
    fn ten_thousand() {
        // BigDecimal doesn't normalize here
        let ten_thousand: BigDecimal = "10000".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&ten_thousand).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 1,
                digits: vec![1]
            }
        );
    }

    #[test]
    fn two_digits() {
        let two_digits: BigDecimal = "12345".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&two_digits).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 1,
                digits: vec![1, 2345]
            }
        );
    }

    #[test]
    fn one_tenth() {
        let one_tenth: BigDecimal = "0.1".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&one_tenth).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 1,
                weight: -1,
                digits: vec![1000]
            }
        );
    }

    #[test]
    fn one_hundredth() {
        let one_hundredth: BigDecimal = "0.01".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&one_hundredth).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 2,
                weight: -1,
                digits: vec![100]
            }
        );
    }

    #[test]
    fn twelve_thousandths() {
        let twelve_thousandths: BigDecimal = "0.012".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&twelve_thousandths).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 3,
                weight: -1,
                digits: vec![120]
            }
        );
    }

    #[test]
    fn decimal_1() {
        let decimal: BigDecimal = "1.2345".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&decimal).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 4,
                weight: 0,
                digits: vec![1, 2345]
            }
        );
    }

    #[test]
    fn decimal_2() {
        let decimal: BigDecimal = "0.12345".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&decimal).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 5,
                weight: -1,
                digits: vec![1234, 5000]
            }
        );
    }

    #[test]
    fn decimal_3() {
        let decimal: BigDecimal = "0.01234".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&decimal).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 5,
                weight: -1,
                digits: vec![0123, 4000]
            }
        );
    }

    #[test]
    fn decimal_4() {
        let decimal: BigDecimal = "12345.67890".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&decimal).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 5,
                weight: 1,
                digits: vec![1, 2345, 6789]
            }
        );
    }

    #[test]
    fn one_digit_decimal() {
        let one_digit_decimal: BigDecimal = "0.00001234".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&one_digit_decimal).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 8,
                weight: -2,
                digits: vec![1234]
            }
        );
    }

    #[test]
    fn issue_423_four_digit() {
        // This is a regression test for https://github.com/launchbadge/sqlx/issues/423
        let four_digit: BigDecimal = "1234".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&four_digit).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 0,
                digits: vec![1234]
            }
        );
    }

    #[test]
    fn issue_423_negative_four_digit() {
        // This is a regression test for https://github.com/launchbadge/sqlx/issues/423
        let negative_four_digit: BigDecimal = "-1234".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&negative_four_digit).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Negative,
                scale: 0,
                weight: 0,
                digits: vec![1234]
            }
        );
    }

    #[test]
    fn issue_423_eight_digit() {
        // This is a regression test for https://github.com/launchbadge/sqlx/issues/423
        let eight_digit: BigDecimal = "12345678".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&eight_digit).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Positive,
                scale: 0,
                weight: 1,
                digits: vec![1234, 5678]
            }
        );
    }

    #[test]
    fn issue_423_negative_eight_digit() {
        // This is a regression test for https://github.com/launchbadge/sqlx/issues/423
        let negative_eight_digit: BigDecimal = "-12345678".parse().unwrap();
        assert_eq!(
            PgNumeric::try_from(&negative_eight_digit).unwrap(),
            PgNumeric::Number {
                sign: PgNumericSign::Negative,
                scale: 0,
                weight: 1,
                digits: vec![1234, 5678]
            }
        );
    }
}

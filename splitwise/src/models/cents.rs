use std::ops::{Add, Mul};

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use color_eyre::eyre::Result;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct Cents(pub i64);

impl Cents {
    pub fn milli_dollars(&self) -> i64 {
        self.0 * 10
    }
}

impl Add for Cents {
    type Output = Cents;

    fn add(self, rhs: Self) -> Self::Output {
        Cents(self.0 + rhs.0)
    }
}

impl Mul for Cents {
    type Output = Cents;

    fn mul(self, rhs: Self) -> Self::Output {
        Cents(self.0 * rhs.0)
    }
}

impl<'de> Deserialize<'de> for Cents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CentsVisitor {}

        impl<'de> Visitor<'de> for CentsVisitor {
            type Value = Cents;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A string representing an amount of mount e.g. 22.11")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let parts: Vec<&str> = value.split('.').collect();

                let dollars = parts[0].parse::<i64>().map_err(E::custom)?;

                // Splitwise will truncate the ones position leading to an incorrect parsing
                // e.g. 5.6 instead of 5.60 leads to 500 + 6 instead of 500 + 60
                let cents = format!("{:0<2}", parts[1]).parse::<i64>().map_err(E::custom)?;

                Ok(Cents(dollars * 100 + dollars.signum() * cents))
            }
        }

        deserializer.deserialize_string(CentsVisitor {})
    }
}

impl Serialize for Cents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let dollars = self.0 / 100;
        let cents = self.0.abs() % 100;

        let result = format!("{}.{:0>2}", dollars, cents);

        serializer.serialize_str(&result)
    }
}

#[cfg(test)]
mod cents_test {
    use quickcheck_macros::quickcheck;
    use color_eyre::eyre::Result;


    use super::Cents;

    #[quickcheck]
    fn test(val1: i32) -> bool {
        let data = serde_json::to_string(&Cents(val1.into())).unwrap();
        let val2: i32 = serde_json::from_str::<Cents>(&data)
            .unwrap()
            .0
            .try_into()
            .unwrap();

        val1 == val2
    }

    #[test]
    fn test_rounding() -> Result<()> {
        let value = serde_json::from_str::<Cents>("\"5.6\"")?;

        assert_eq!(Cents(560), value);

        Ok(())
    }
}

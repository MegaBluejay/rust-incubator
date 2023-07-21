use std::num::TryFromIntError;

use thiserror::Error;
use time::{error::ComponentRange, Date};

fn main() {
    println!("Implement me!");
}

const NOW: &str = "2019-06-26";

struct User {
    birthday: Date,
}

#[derive(Debug, Error)]
enum UserError {
    #[error("invalid date")]
    ComponentRange(#[from] ComponentRange),
    #[error("date component not in range")]
    TryFromIntError(#[from] TryFromIntError),
}

impl User {
    fn with_birthdate(year: i32, month: u32, day: u32) -> Result<Self, UserError> {
        Ok(Self {
            birthday: Date::from_calendar_date(
                year,
                u8::try_from(month)?.try_into()?,
                day.try_into()?,
            )?,
        })
    }

    /// Returns current age of [`User`] in years.
    fn age(&self) -> u16 {
        unimplemented!()
    }

    /// Checks if [`User`] is 18 years old at the moment.
    fn is_adult(&self) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod age_spec {
    use super::*;

    #[test]
    fn counts_age() -> Result<(), UserError> {
        for ((y, m, d), expected) in [
            ((1990, 6, 4), 29),
            ((1990, 7, 4), 28),
            ((0, 1, 1), 2019),
            ((1970, 1, 1), 49),
            ((2019, 6, 25), 0),
        ] {
            let user = User::with_birthdate(y, m, d)?;
            assert_eq!(user.age(), expected);
        }
        Ok(())
    }

    #[test]
    fn zero_if_birthdate_in_future() -> Result<(), UserError> {
        for ((y, m, d), expected) in [
            ((2032, 6, 25), 0),
            ((2016, 6, 27), 0),
            ((3000, 6, 27), 0),
            ((9999, 6, 27), 0),
        ] {
            let user = User::with_birthdate(y, m, d)?;
            assert_eq!(user.age(), expected);
        }
        Ok(())
    }
}

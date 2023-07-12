use crate::{
    email::{EmailError, EmailStr, EmailString},
    random::Random,
};

mod email {
    use std::{borrow::Borrow, fmt::Display, ops::Deref};

    use once_cell::sync::Lazy;
    use ref_cast::RefCast;
    use regex::Regex;

    static EMAIL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\w+([-+.']\w+)*@\w+([-.]\w+)*\.\w+([-.]\w+)*$").unwrap());

    #[derive(Debug)]
    #[repr(transparent)]
    pub struct EmailString(String);

    #[derive(RefCast, Debug)]
    #[repr(transparent)]
    pub struct EmailStr(str);

    #[derive(Debug)]
    pub enum EmailError {
        InvalidEmail,
    }

    impl Display for EmailStr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", &self.0)
        }
    }

    impl Display for EmailString {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", &self.0)
        }
    }

    impl<'a> TryFrom<&'a str> for &'a EmailStr {
        type Error = EmailError;

        fn try_from(value: &'a str) -> Result<Self, Self::Error> {
            if EMAIL_REGEX.is_match(value) {
                Ok(EmailStr::ref_cast(value))
            } else {
                Err(EmailError::InvalidEmail)
            }
        }
    }

    impl AsRef<EmailStr> for EmailStr {
        fn as_ref(&self) -> &EmailStr {
            self
        }
    }

    impl AsRef<str> for EmailStr {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    impl From<&EmailStr> for EmailString {
        fn from(value: &EmailStr) -> Self {
            Self(value.0.into())
        }
    }

    impl ToOwned for EmailStr {
        type Owned = EmailString;

        fn to_owned(&self) -> Self::Owned {
            self.into()
        }
    }

    impl Borrow<EmailStr> for EmailString {
        fn borrow(&self) -> &EmailStr {
            self
        }
    }

    impl TryFrom<String> for EmailString {
        type Error = EmailError;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            match TryInto::<&EmailStr>::try_into(value.as_str()) {
                Ok(_) => Ok(Self(value)),
                Err(err) => Err(err),
            }
        }
    }

    impl Deref for EmailString {
        type Target = EmailStr;

        fn deref(&self) -> &Self::Target {
            EmailStr::ref_cast(&self.0)
        }
    }

    impl AsRef<EmailStr> for EmailString {
        fn as_ref(&self) -> &EmailStr {
            self
        }
    }

    impl AsRef<str> for EmailString {
        fn as_ref(&self) -> &str {
            self.deref().as_ref()
        }
    }
}

mod random {
    use std::{
        borrow::{Borrow, BorrowMut},
        ops::{Deref, DerefMut},
    };

    use rand::Rng;

    pub struct Random<T>([T; 3]);

    impl<T> Random<T> {
        pub fn new(x: T, y: T, z: T) -> Self {
            Self([x, y, z])
        }
    }

    impl<T> Deref for Random<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0[rand::thread_rng().gen_range(0..3)]
        }
    }

    impl<T> DerefMut for Random<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0[rand::thread_rng().gen_range(0..3)]
        }
    }

    impl<T> Borrow<T> for Random<T> {
        fn borrow(&self) -> &T {
            self
        }
    }

    impl<T> BorrowMut<T> for Random<T> {
        fn borrow_mut(&mut self) -> &mut T {
            self
        }
    }

    impl<T> AsRef<T> for Random<T> {
        fn as_ref(&self) -> &T {
            self
        }
    }

    impl<T> AsMut<T> for Random<T> {
        fn as_mut(&mut self) -> &mut T {
            self
        }
    }
}

fn main() {
    let invalid: Result<&EmailStr, EmailError> = "something".try_into();
    let raw_email_str: &str = "example@email.com";
    let email_str: &EmailStr = raw_email_str.try_into().unwrap();
    let email_string1: EmailString = email_str.to_owned();
    let email_string2: EmailString = raw_email_str.to_owned().try_into().unwrap();
    println!(
        "{:?}, {}, {}, {}",
        invalid, email_str, email_string1, email_string2
    );

    let mut rand = Random::new(1, 2, 3);
    *rand = 10;
    println!("{}, {}, {}", *rand, *rand, *rand);
}

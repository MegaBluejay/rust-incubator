use std::{
    cmp::Ordering,
    env,
    io::{self, BufRead},
};

fn main() {
    println!("Guess the number!");

    let secret_number = get_secret_number();

    loop {
        println!("Please input your guess.");

        let guess = match get_guess_number() {
            Some(n) => n,
            _ => continue,
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}

fn get_secret_number() -> u32 {
    get_secret_number_from(env::args())
}

fn get_secret_number_from(args: impl IntoIterator<Item = String>) -> u32 {
    let secret_number = args
        .into_iter()
        .skip(1)
        .take(1)
        .last()
        .expect("No secret number is specified");
    secret_number
        .trim()
        .parse()
        .expect("Secret number is not a number")
}

fn get_guess_number() -> Option<u32> {
    get_guess_number_from(io::stdin().lock())
}

fn get_guess_number_from(mut input: impl BufRead) -> Option<u32> {
    let mut guess = String::new();
    input.read_line(&mut guess).expect("Failed to read line");
    guess.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use proptest::collection::size_range;
    use proptest::prelude::*;

    use super::*;

    #[test]
    #[should_panic = "No secret number"]
    fn no_secret_number() {
        get_secret_number_from(vec!["".to_owned()]);
    }

    #[test]
    #[should_panic = "not a number"]
    fn secret_number_not_a_number() {
        get_secret_number_from(vec!["".to_owned(), "a".to_owned()]);
    }

    #[test]
    fn basic_secret_number() {
        assert_eq!(
            5,
            get_secret_number_from(vec!["".to_owned(), "5".to_owned()])
        );
    }

    #[test]
    fn basic_guess_number() {
        assert_eq!(Some(6), get_guess_number_from("6".as_bytes()));
    }

    #[test]
    fn invalid_guess_number() {
        assert_eq!(None, get_guess_number_from(&[][..]));
    }

    proptest! {
        #[test]
        fn valid_secret_number(
            n: u32,
            before in r"\s*",
            after in r"\s*",
            mut args in any_with::<Vec<String>>((size_range(1..5), Default::default()))
        ) {
            let input = format!("{}{}{}", before, n, after);
            args.insert(1, input);
            prop_assert_eq!(n, get_secret_number_from(args));
        }

        #[test]
        fn valid_guess_number(
            n: u32,
            before in r"[\s--\n]*",
            after in r"[\s--\n]*",
            rest: String,
        ) {
            let input = format!("{}{}{}{}{}", before, n, after, "\n", rest);
            prop_assert_eq!(Some(n), get_guess_number_from(input.as_bytes()))
        }

        #[test]
        fn guess_number_no_panic(input: String) {
            let _ = get_guess_number_from(format!("{}{}", input, "\n").as_bytes());
        }
    }
}

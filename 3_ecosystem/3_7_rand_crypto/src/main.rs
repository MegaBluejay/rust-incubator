use std::{collections::HashSet, fs::read, io, path::Path};

use argon2::{
    password_hash::{PasswordHashString, SaltString},
    Argon2, PasswordHasher,
};
use rand::{
    distributions::{Alphanumeric, DistString, Slice},
    prelude::Distribution,
    seq::SliceRandom,
    thread_rng,
};
use sha3::{Digest, Sha3_256};

fn generate_password(len: usize, symbols: impl AsRef<[char]>) -> String {
    let symbols = symbols.as_ref();
    debug_assert_eq!(symbols.iter().collect::<HashSet<_>>().len(), symbols.len());
    let dist = Slice::new(symbols).unwrap();
    let mut pass = String::with_capacity(len);
    pass.extend(dist.sample_iter(&mut thread_rng()).take(len));
    pass
}

fn select_rand_val<T>(vals: &[T]) -> Option<&T> {
    vals.choose(&mut thread_rng())
}

fn new_access_token() -> String {
    Alphanumeric.sample_string(&mut thread_rng(), 64)
}

fn get_file_hash(path: impl AsRef<Path>) -> io::Result<String> {
    Ok(hex::encode(<Sha3_256 as Digest>::digest(read(
        path.as_ref(),
    )?)))
}

fn hash_password(password: impl AsRef<str>) -> PasswordHashString {
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_ref().as_bytes(), &salt)
        .unwrap()
        .into()
}

fn main() {
    dbg!(generate_password(10, ['a', 'b', 'c']));

    dbg!(select_rand_val(&[1, 2, 3]));

    dbg!(new_access_token());

    dbg!(get_file_hash("README.md").unwrap());

    dbg!(hash_password("password"));
}

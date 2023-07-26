use std::{fs::read, path::Path};

use argon2::{
    password_hash::{PasswordHashString, SaltString},
    Argon2, PasswordHasher,
};
use rand::{
    distributions::{Alphanumeric, DistString, Slice},
    prelude::Distribution,
    rngs::OsRng,
    seq::SliceRandom,
    thread_rng,
};
use sha3::{Digest, Sha3_256};

fn generate_password(len: usize, symbols: impl AsRef<[char]>) -> String {
    let dist = Slice::new(symbols.as_ref()).unwrap();
    let mut pass = String::with_capacity(len);
    pass.extend(dist.sample_iter(&mut thread_rng()).take(len));
    pass
}

fn select_rand_val<T>(vals: &[T]) -> &T {
    vals.choose(&mut thread_rng()).unwrap()
}

fn new_access_token() -> String {
    Alphanumeric.sample_string(&mut thread_rng(), 64)
}

fn get_file_hash(path: impl AsRef<Path>) -> String {
    hex::encode(<Sha3_256 as Digest>::digest(read(path.as_ref()).unwrap()))
}

fn hash_password(password: impl AsRef<str>) -> PasswordHashString {
    let salt = SaltString::generate(&mut OsRng);
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

    dbg!(get_file_hash("README.md"));

    dbg!(hash_password("password"));
}

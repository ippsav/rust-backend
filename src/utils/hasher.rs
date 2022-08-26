use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, Result, SaltString,
    },
    Argon2,
};
pub fn hash_password(password: &[u8]) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)

    Ok(argon2.hash_password(password, &salt)?.to_string())
}

pub fn verify_password(password: &[u8], password_hash: &str) -> Result<bool> {
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Verify password against PHC string.
    //
    // NOTE: hash params from `parsed_hash` are used instead of what is configured in the
    // `Argon2` instance.
    let parsed_hash = PasswordHash::new(password_hash)?;

    Ok(argon2.verify_password(password, &parsed_hash).is_ok())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_and_verify_correct_password() {
        let password = "it_should_work";

        let hashed_password = hash_password(password.as_bytes()).unwrap();

        dbg!(&hashed_password, hashed_password.len());

        assert!(verify_password(password.as_bytes(), hashed_password.as_str()).unwrap())
    }

    #[test]
    fn hash_and_verify_wrong_password() {
        let password = "it_should_not_work";

        let hashed_password = dbg!(hash_password(password.as_bytes()).unwrap());
        dbg!(&hashed_password, hashed_password.len());

        assert!(!verify_password("wrong_password".as_bytes(), hashed_password.as_str()).unwrap())
    }
}

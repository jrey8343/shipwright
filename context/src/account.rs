use argon2::{
    Argon2, PasswordHasher,
    password_hash::{self, SaltString, rand_core::OsRng},
};
use shipwright_db::{
    entities::user::{User, UserCredentials, UserStatus},
    Error as DbError,
};

/// The Account context handles all business logic related to user accounts,
/// authentication, and sessions without directly interacting with the database.
pub struct Account;

impl Account {
    /// Generates a password hash using Argon2.
    pub fn generate_password_hash(password: &str) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hashed_password = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(hashed_password)
    }

    /// Validates user credentials against a user record.
    pub fn validate_credentials(user: &User, credentials: &UserCredentials) -> Result<(), DbError> {
        if user.status != UserStatus::Confirmed {
            return Err(DbError::ValidationError(
                validator::ValidationErrors::new(),
            ));
        }

        let argon2 = Argon2::default();
        let parsed_hash = password_hash::PasswordHash::new(&user.password_hash)
            .map_err(|_| DbError::PasswordHashError(password_hash::Error::Password))?;

        argon2
            .verify_password(credentials.password.as_bytes(), &parsed_hash)
            .map_err(|_| DbError::PasswordHashError(password_hash::Error::Password))?;

        Ok(())
    }

    /// Validates a user's registration data.
    pub fn validate_registration(credentials: &UserCredentials) -> Result<(), DbError> {
        credentials.validate().map_err(DbError::ValidationError)
    }
} 
use failure::Error;
use rand_pcg::rand_core::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub hash_session: Vec<u8>,
    pub email_validated: bool,
}

impl UserInfo {
    pub fn new_user(username: String, email: String, password: String) -> Result<(), Error> {
        let mut password_salt = vec![0u8; 64];
        use chrono::prelude::*;
        let timestamp = Utc::now().timestamp_nanos();

        rand_pcg::Mcg128Xsl64::new(0xcafef00dd15ea5e5 + timestamp as u128)
            .fill_bytes(&mut password_salt);

        let hash_session = argon2rs::verifier::Encoded::new(
            argon2rs::Argon2::default(argon2rs::Variant::Argon2i),
            password.as_bytes(),
            &password_salt,
            &[],
            &[],
        );

        ensure!( password.len() >= 8, "Password must be at least 10 characters long");
        let all_chars_ok = username.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');

        ensure!(username.len() > 0, "User name cannot be empty");
        ensure!(all_chars_ok, "Username contains invalid characters");
        ensure!(!UserInfo::user_dir(&username).exists(), "User already exists");

        // TODO - Check that email address is not in use by any other user
        // TODO - Perform password strength validation
        // TODO - Perform username String validation
        // TODO - Check username doesn't exist yet


        std::fs::create_dir_all(&UserInfo::user_dir(&username))?;

        let userinfo = UserInfo {
            username: username.clone(),
            email,
            hash_session: hash_session.to_u8(),
            email_validated: false,
        };
        let userinfo_json = serde_json::to_string(&userinfo).unwrap();

        if let Err(x) = std::fs::write(UserInfo::user_file(&username), &userinfo_json) {
            error!("Error trying to save user info: {}", x);
            return Err(x.into());
        }

        Ok(())
    }

    fn user_dir(username: &str) -> std::path::PathBuf {
        std::path::Path::new("users").join(username)
    }

    fn user_file(username: &str) -> std::path::PathBuf {
        UserInfo::user_dir(username).join("userinfo.json")
    }

    pub fn load_user_data(username: &str) -> Result<UserInfo, Error> {
        let json_data =
            std::fs::read_to_string(&UserInfo::user_file(username))?;
        Ok(serde_json::from_str(&json_data)?)
    }

    pub fn verify_password(&self, password: String) -> bool {
        let hash_session = match argon2rs::verifier::Encoded::from_u8(&self.hash_session) {
            Err(x) => {
                error!("Error trying to reload hashing session: {}", x);
                return false;
            }
            Ok(x) => x,
        };

        hash_session.verify(password.as_bytes())
    }
}

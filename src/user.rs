use failure::Error;
use serde::{Deserialize, Serialize};
use rand_pcg::rand_core::RngCore;

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub hash_session: Vec<u8>,
    pub email_validated: bool,
}

impl UserInfo {

    fn new_user(username: String, email: String, password: String) -> Result<(), Error>{

        let mut password_salt = vec![0u8; 64];
        rand_pcg::Mcg128Xsl64::new(0xcafef00dd15ea5e5).fill_bytes(&mut password_salt);

        let hash_session = argon2rs::verifier::Encoded::new(
            argon2rs::Argon2::default(argon2rs::Variant::Argon2i),
            password.as_bytes(), 
            &password_salt,
            &[],
            &[],
        );

        // TODO - Check that email address is not in use by any other user
        // TODO - Validate email
        // TODO - Perform password strength validation
        // TODO - Perform username String validation
        // TODO - Check username doesn't exist yet
        
        let filename = format!("pins/{}/userinfo.json", username);

        let userinfo = UserInfo {
            username,
            email,
            hash_session: hash_session.to_u8(),
            email_validated: false,
        };
        let userinfo_json = serde_json::to_string(&userinfo).unwrap();

        if let Err(x) = std::fs::write(filename, &userinfo_json){
            // TODO - Log failure
            return Err(x.into());
        }
        
        Ok(())
    }

    fn load_user_data(username: String) -> Result<UserInfo, Error> {

        // TODO
        Err(failure::err_msg("Unimplemented"))
    }

    fn verify_password(&self, password: String) -> bool {
        let hash_session = match argon2rs::verifier::Encoded::from_u8(&self.hash_session) {
            Err(x) => {
                // TODO log hashing session error
                return false;
            }
            Ok(x) => x,
        };

        hash_session.verify(password.as_bytes())
    }
}

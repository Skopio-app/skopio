pub struct Keyring;

impl Keyring {
    pub fn get_password(service: &str, user: &str) -> keyring::Result<Option<String>> {
        Ok(keyring::Entry::new(service, user)?.get_password().ok())
    }

    pub fn set_password(service: &str, user: &str, password: &str) -> keyring::Result<()> {
        keyring::Entry::new(service, user)?.set_password(password)?;
        Ok(())
    }

    pub fn get_or_set_password(
        service: &str,
        user: &str,
        password: &str,
    ) -> keyring::Result<String> {
        let _password = Self::get_password(service, user)?;
        match _password {
            Some(key) => Ok(key),
            None => {
                Self::set_password(service, user, password)?;
                Ok(password.to_string())
            }
        }
    }

    pub fn delete_password(service: &str, user: &str) -> keyring::Result<()> {
        keyring::Entry::new(service, user)?.delete_credential()?;
        Ok(())
    }

    pub fn get_secret(service: &str, user: &str) -> keyring::Result<Option<Vec<u8>>> {
        Ok(keyring::Entry::new(service, user)?.get_secret().ok())
    }

    pub fn set_secret(service: &str, user: &str, secret: &[u8]) -> keyring::Result<()> {
        keyring::Entry::new(service, user)?.set_secret(secret)
    }

    pub fn get_or_set_secret(service: &str, user: &str, secret: &[u8]) -> keyring::Result<Vec<u8>> {
        let _secret = Self::get_secret(service, user)?;
        match _secret {
            Some(key) => Ok(key),
            None => {
                Self::set_secret(service, user, secret)?;
                Ok(secret.to_vec())
            }
        }
    }

    pub fn delete_secret(service: &str, user: &str) -> keyring::Result<()> {
        keyring::Entry::new(service, user)?.delete_credential()
    }
}

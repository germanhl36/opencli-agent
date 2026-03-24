use crate::error::OpenCLIError;

pub fn store_api_key(provider: &str, key: &str) -> Result<(), OpenCLIError> {
    let entry = keyring::Entry::new("opencli", provider)
        .map_err(|e| OpenCLIError::Keychain(e.to_string()))?;
    entry.set_password(key)
        .map_err(|e| OpenCLIError::Keychain(e.to_string()))
}

pub fn get_api_key(provider: &str) -> Result<Option<String>, OpenCLIError> {
    let entry = keyring::Entry::new("opencli", provider)
        .map_err(|e| OpenCLIError::Keychain(e.to_string()))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(OpenCLIError::Keychain(e.to_string())),
    }
}

pub fn delete_api_key(provider: &str) -> Result<(), OpenCLIError> {
    let entry = keyring::Entry::new("opencli", provider)
        .map_err(|e| OpenCLIError::Keychain(e.to_string()))?;
    entry.delete_credential()
        .map_err(|e| OpenCLIError::Keychain(e.to_string()))
}

use super::CreateUserError;

pub(super) fn validate_username(username: String) -> Result<String, CreateUserError> {
    let username = username.trim();
    if !(3..=32).contains(&username.len()) {
        return Err(CreateUserError::InvalidUsername);
    }

    if !username
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(CreateUserError::InvalidUsername);
    }

    Ok(username.to_owned())
}

pub(super) fn normalize_username(username: String) -> String {
    username.trim().to_ascii_uppercase()
}

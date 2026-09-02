use super::CreateUserError;

pub(super) fn validate_username(username: &str) -> Result<&str, CreateUserError> {
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

    Ok(username)
}

pub(super) fn validate_password(password: &str) -> Result<&str, CreateUserError> {
    let password = password.trim();
    let length = password.chars().count();

    if !(12..=256).contains(&length) {
        return Err(CreateUserError::InvalidPassword);
    }

    Ok(password)
}

pub(super) fn normalize_username(username: &str) -> String  {
    username.trim().to_ascii_lowercase()
}


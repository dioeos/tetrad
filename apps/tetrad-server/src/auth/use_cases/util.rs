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

pub(super) fn normalize_username(username: &str) -> String {
    username.trim().to_ascii_lowercase()
}

#[cfg(not(test))]
mod tests {
    use super::*;

    #[test]
    fn validate_username_returns_invalid_username_when_too_short() {
        let err = validate_username("me");
        assert!(matches!(err, Err(CreateUserError::InvalidUsername)));
    }

    #[test]
    fn validate_username_returns_invalid_username_when_too_long() {
        let x = "a";
        let long = x.repeat(33);
        let err = validate_username(&long);
        assert!(matches!(err, Err(CreateUserError::InvalidUsername)));
    }

    #[test]
    fn validate_username_returns_invalid_username_when_not_all_ascii_alphanumeric_or_underscores_or_hyphens()
     {
        let err1 = validate_username("user.name");
        let err2 = validate_username("user name");
        assert!(matches!(err1, Err(CreateUserError::InvalidUsername)));
        assert!(matches!(err2, Err(CreateUserError::InvalidUsername)));
    }

    #[test]
    fn validate_password_returns_invalid_when_too_short() {
        let err = validate_password("a");
        assert!(matches!(err, Err(CreateUserError::InvalidPassword)));
    }

    #[test]
    fn validate_password_returns_invalid_when_too_long() {
        let x = "a";
        let long = x.repeat(257);
        let err = validate_password(&long);
        assert!(matches!(err, Err(CreateUserError::InvalidPassword)));
    }
}

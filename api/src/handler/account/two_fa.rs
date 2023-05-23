use anyhow::Error;

// 2FA
// account.confirmPasswordEmail
pub async fn account_confirm_password_email() -> Result<i32, Error> {
    Ok(1)
}

// account.resendPasswordEmail
pub async fn account_resend_password_email() -> Result<i32, Error> {
    Ok(1)
}

// account.cancelPasswordEmail
pub async fn account_cancel_password_email() -> Result<i32, Error> {
    Ok(1)
}

// account.getPassword
pub async fn account_get_password() -> Result<i32, Error> {
    Ok(1)
}

// account.getPasswordSettings
pub async fn account_get_password_settings() -> Result<i32, Error> {
    Ok(1)
}

// account.updatePasswordSettings
pub async fn account_update_password_settings() -> Result<i32, Error> {
    Ok(1)
}

// account.declinePasswordReset
pub async fn account_decline_password_reset() -> Result<i32, Error> {
    Ok(1)
}
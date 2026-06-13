use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UserRegister {
    pub account: String,
    pub email: String,
    pub password: String,
    pub shipping_address: Option<String>,
    pub name: Option<String>,
    pub recipient_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserRegisterResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserRefreshToken {
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserRefreshTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserLogout {
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserLogoutResponse {
    pub msg: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserDeleteResponse {
    pub msg: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserResetPasswordEmail {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserResetPasswordEmailResponse {
    pub msg: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserResetPassword {
    pub token: String,
    pub password: String,
}
#[derive(Deserialize, Serialize)]
pub struct UserResetPasswordResponse {
    pub msg: String,
}

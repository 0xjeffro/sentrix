use base64::{Engine, engine::general_purpose};
use chrono::{Duration, Utc};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthToken {
    pub user: String, // User ID
    pub exp: u64,     // Expiration time in seconds
    pub qps: u32,     // Queries per second

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sig: Option<String>,
}

#[derive(Debug)]
pub enum TokenError {
    SerializationError,
    InvalidSecret,
    DecodeError,
    InvalidSignature,
    MissingSignature,
}

impl AuthToken {
    pub fn signable_string(&self) -> Result<String, TokenError> {
        #[derive(Serialize)]
        struct SignableToken<'a> {
            user: &'a str,
            exp: u64,
            qps: u32,
        }
        let s = SignableToken {
            user: &self.user,
            exp: self.exp,
            qps: self.qps,
        };
        serde_json::to_string(&s).map_err(|_| TokenError::SerializationError)
    }

    pub fn compute_signature(&mut self, secret: &str, fill: bool) -> Result<String, TokenError> {
        let signable_str = self.signable_string()?;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|_| TokenError::InvalidSecret)?;
        mac.update(signable_str.as_bytes());
        let sig = mac.finalize().into_bytes();
        let sig_str = general_purpose::URL_SAFE_NO_PAD.encode(&sig);
        if fill {
            self.sig = Some(sig_str.clone());
        }
        Ok(sig_str)
    }

    pub fn generate_token(&self) -> Result<String, TokenError> {
        let token_str = serde_json::to_string(self).map_err(|_| TokenError::SerializationError)?;
        let encoded_token = general_purpose::URL_SAFE_NO_PAD.encode(token_str);
        Ok(encoded_token)
    }
}

pub fn generate_token(secret: &str, user: &str, qps: u32, ttl_secs: u64) -> String {
    let expiration = (Utc::now() + Duration::seconds(ttl_secs as i64)).timestamp() as u64;
    let mut raw_token = AuthToken {
        user: user.to_string(),
        exp: expiration,
        qps,
        sig: None,
    };
    #[cfg(debug_assertions)]
    {
        let _json_str = raw_token.signable_string().unwrap();
        println!("Raw token JSON: {}", _json_str);
    }
    raw_token
        .compute_signature(secret, true)
        .unwrap_or_else(|err| panic!("Failed to compute signature: {:?}", err));

    #[cfg(debug_assertions)]
    println!("Raw token: {:?}", raw_token);
    #[cfg(debug_assertions)]
    println!("Signature: {}", raw_token.sig.as_ref().unwrap());
    let encoded_token = raw_token.generate_token().unwrap();
    #[cfg(debug_assertions)]
    println!("Generated token: {}", encoded_token);
    let decoded_token = general_purpose::URL_SAFE_NO_PAD
        .decode(encoded_token.clone())
        .unwrap();
    #[cfg(debug_assertions)]
    {
        let decoded_str = String::from_utf8(decoded_token).unwrap();
        println!("Decoded token: {}", decoded_str);
    }
    encoded_token
}

pub fn verify_token(token: &str, secret: &str) -> Result<AuthToken, TokenError> {
    let decoded_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| TokenError::DecodeError)?;
    let mut auth_token: AuthToken =
        serde_json::from_slice(&decoded_bytes).map_err(|_| TokenError::DecodeError)?;

    let sig = auth_token
        .sig
        .as_ref()
        .ok_or(TokenError::MissingSignature)?
        .to_string();
    let expected_sig = auth_token.compute_signature(secret, false)?;

    if sig != expected_sig {
        return Err(TokenError::InvalidSignature);
    }
    Ok(auth_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        use crate::config::Settings;
        let settings = Settings::new().unwrap();
        let secret = settings.app.secret_key.clone();
        let token = generate_token(&secret, "jeffro", 1, 1800);

        println!("Generated token: {}", token);
    }

    #[test]
    fn test_generate_and_verify_token() {
        use crate::config::Settings;
        let settings = Settings::new().unwrap();
        let secret = settings.app.secret_key.clone();
        let token = generate_token(&secret, "jeffro", 100, 1800);

        println!("Generated token: {}", token);

        let verified_token = verify_token(&token, &secret).unwrap();
        println!("Verified token: {:?}", verified_token);
    }
}

use std::sync::Mutex;
use serde::Deserialize;
use crate::framebufferd::shared::do_get_or_patch_request_to_socket;

#[derive(Clone, Deserialize)]
struct Token {
    pub token: String,
    pub expiration: u64,
}

static TOKEN: Mutex<Option<Token>> = Mutex::new(None);

fn handle_token_response(data: Vec<u8>, status: u16) -> Option<Token> {
    if status != 200 {
        return None;
    }

    let token_response: Token = match serde_json::from_slice(&data) {
        Ok(tr) => tr,
        Err(_) => return None,
    };
    Some(token_response)
}

#[derive(Deserialize)]
struct TokenRenewResponse {
    pub new_expiration: u64,
}

fn handle_renew_response(data: Vec<u8>, status: u16) -> Option<TokenRenewResponse> {
    if status != 200 {
        return None;
    }

    let renew_response: TokenRenewResponse = match serde_json::from_slice(&data) {
        Ok(rr) => rr,
        Err(_) => return None,
    };
    Some(renew_response)
}

pub fn get_token() -> Result<Option<String>, ()> {
    let mut token_guard = TOKEN.lock().unwrap();

    if let Some(token) = &*token_guard {
        if token.expiration > chrono::Utc::now().timestamp_millis() as u64 {
            return Ok(Some(token.token.clone()));
        } else {
            // Token expired, need to fetch a new one.
            return match do_get_or_patch_request_to_socket(&format!("/renew?token={}", token.token), true, None) {
                None => Err(()),
                Some((data, status, _)) => Ok(match handle_renew_response(data, status) {
                    None => None,
                    Some(new_token) => {
                        let t = token.token.clone();
                        *token_guard = Some(Token {
                            token: t.clone(),
                            expiration: new_token.new_expiration,
                        });
                        Some(t)
                    },
                }),
            };
        }
    }

    match do_get_or_patch_request_to_socket("/authorize", false, None) {
        None => Err(()),
        Some((data, status, _)) => Ok(match handle_token_response(data, status) {
            None => None,
            Some(new_token) => {
                *token_guard = Some(new_token.clone());
                Some(new_token.token)
            },
        }),
    }
}

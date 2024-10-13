use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionData {
    pub nonce: Option<String>,
    pub state: Option<String>,
    pub client_id: Option<String>,
    pub state_token: Option<String>,
    pub bearer: Option<String>,
    pub cookies: Option<HashMap<String, String>>,
}
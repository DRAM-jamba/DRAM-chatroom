use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String
}
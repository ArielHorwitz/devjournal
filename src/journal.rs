use serde::{Deserialize, Serialize};
use polars::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Journal {
    pub name: String,
    pub password: String,
    
}


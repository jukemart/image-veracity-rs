use eyre::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::hash::VeracityHash;
use trillian_client::client::TrillianClient;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub trillian: TrillianClient,
}

impl AppState {
    pub async fn new(host: String) -> Result<AppState> {
        let trillian = TrillianClient::new(host).await?.build();
        Ok(AppState { trillian })
    }
}

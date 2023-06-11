use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::hash::VeracityHash;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub todos: Arc<Mutex<HashMap<Uuid, VeracityHash>>>,
}

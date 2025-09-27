use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use crate::network::client::Client;

pub type ClientPool = Arc<RwLock<HashMap<u64, Arc<Client>>>>;

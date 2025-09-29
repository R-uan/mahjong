use crate::network::client::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type ClientPool = Arc<RwLock<HashMap<i32, Arc<Client>>>>;

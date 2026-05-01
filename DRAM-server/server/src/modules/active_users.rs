use std::{collections::{HashMap}, sync::Arc};

use tokio::sync::{RwLock};


pub type UsersMap = Arc<RwLock<HashMap<String, String>>>;
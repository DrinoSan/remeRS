use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    // id: String,
    pub time: SystemTime,
    pub subject: String,
    pub already_dispatched: bool,
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "$kind")]
pub enum Metadata {
    None,
    String(String),
    KeyValues(HashMap<String, Metadata>),
}

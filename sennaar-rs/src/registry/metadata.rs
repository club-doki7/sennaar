use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[derive(JsonSchema)]
#[serde(tag = "$kind")]
pub enum Metadata {
    None,
    String { value: String },
    KeyValues { kvs: HashMap<String, Metadata> },
}

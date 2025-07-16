use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "$kind")]
pub enum Metadata {
    None,
    String(String),
    KeyValues(HashMap<String, Metadata>),
}

pub trait Entity : Eq + Ord {
    fn entity_name(&self) -> &Identifier;
    fn entity_metadata(&self) -> &HashMap<String, Metadata>;
    fn entity_metadata_mut(&mut self) -> &mut HashMap<String, Metadata>;
    fn entity_doc(&self) -> &[String];
    fn entity_doc_mut(&mut self) -> &mut Vec<String>;

    fn has_metadata(&self, key: &str) -> bool {
        self.entity_metadata().contains_key(key)
    }

    fn try_get_metadata(&self, key: &str) -> Option<&Metadata> {
        self.entity_metadata().get(key)
    }

    fn get_metadata(&self, key: &str) -> &Metadata {
        self.try_get_metadata(key).unwrap()
    }
}

macro_rules! entity {
    ($name:ident, $($field:ident: $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub name: crate::Identifier,
            pub metadata: HashMap<String, Metadata>,
            pub doc: Vec<String>,
            $(pub $field: $type),*
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.name.cmp(&other.name)
            }
        }

        impl Entity for $name {
            fn entity_name(&self) -> &Identifier {
                &self.name
            }

            fn entity_metadata(&self) -> &HashMap<String, Metadata> {
                &self.metadata
            }

            fn entity_metadata_mut(&mut self) -> &mut HashMap<String, Metadata> {
                &mut self.metadata
            }

            fn entity_doc(&self) -> &[String] {
                &self.doc
            }

            fn entity_doc_mut(&mut self) -> &mut Vec<String> {
                &mut self.doc
            }
        }
    };
}

entity!{
    Typedef,
    target: Identifier,
}

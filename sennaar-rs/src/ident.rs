use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Result as FmtResult;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


struct IdentInternal {
    original: Box<str>,
    renamed: OnceCell<Box<str>>,
}

thread_local! {
    static IDENTIFIERS: RefCell<HashMap<String, Rc<IdentInternal>>> =
        RefCell::new(HashMap::new());
}

/// Clear internal identifier renames state.
///
/// Existing identifier renames will not be affected, but new identifiers will not have the same
/// renames -- In other words, they are un-linked. Be cautious about the logical implications of
/// this operation.
pub fn reset_identifier_renames() {
    IDENTIFIERS.with(|renames| {
        renames.borrow_mut().clear();
    });
}

#[derive(Clone)]
pub struct Identifier(Rc<IdentInternal>);

impl JsonSchema for Identifier {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Identifier")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": "string" })
    }
}

impl Identifier {
    pub fn original(&self) -> &str {
        &self.0.original
    }

    pub fn value<'a>(&'a self) -> &'a str {
        self.renamed().unwrap_or(&self.0.original)
    }

    pub fn renamed<'a>(&'a self) -> Option<&'a str> {
        self.0.renamed
            .get()
            .map(|s| s.as_ref())
    }

    pub fn try_rename(&self, new_name: &str) -> Result<(), String> {
        if new_name.contains(':') {
            return Err("Renamed identifiers cannot contain ':'".to_string());
        }

        if let Some(current) = self.renamed() {
            if current == new_name {
                Ok(())
            } else {
                Err(format!(
                    "Identifier '{}' is already renamed to '{}'",
                    self.0.original, current
                ))
            }
        } else {
            let r = self.0.renamed.set(Box::from(new_name));
            Ok(unsafe { r.unwrap_unchecked() })
        }
    }

    pub fn rename(&self, new_name: &str) {
        self.try_rename(new_name).unwrap()
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_addr = Rc::as_ptr(&self.0);
        let other_addr = Rc::as_ptr(&other.0);
        if self_addr == other_addr {
            return Ordering::Equal;
        }

        self.0.original.cmp(&other.0.original)
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(renamed) = self.renamed() {
            write!(f, "Identifier({} -> {})", self.0.original, renamed)
        } else {
            write!(f, "Identifier({})", self.0.original)
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value())
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(renamed) = self.renamed() {
            serializer.serialize_str(&format!("{}:{}", self.0.original, renamed))
        } else {
            serializer.serialize_str(&self.0.original)
        }
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if let Some((original, renamed)) = s.split_once(':') {
            let ident = original.interned();
            match ident.try_rename(renamed) {
                Ok(_) => Ok(ident),
                Err(e) => {
                    Err(DeserializeError::custom(format!(
                        "Failed renaming identifier '{}' to '{}': {}",
                        original, renamed, e
                    )))
                }
            }
        } else {
            Ok(s.interned())
        }
    }
}

pub trait Internalize {
    fn interned(&self) -> Identifier;
}

impl Internalize for str {
    fn interned(&self) -> Identifier {
        assert!(!self.contains(':'), "Identifiers cannot contain ':' in {}", self);

        IDENTIFIERS.with(|renames| {
            let mut renames = renames.borrow_mut();
            let internal = if let Some(state) = renames.get(self) {
                state.clone()
            } else {
                let state = Rc::new(IdentInternal {
                    original: Box::from(self),
                    renamed: OnceCell::new(),
                });
                renames.insert(self.to_string(), state.clone());
                state
            };
            Identifier(internal)
        })
    }
}

impl<T: AsRef<str>> Internalize for T {
    fn interned(&self) -> Identifier {
        self.as_ref().interned()
    }
}

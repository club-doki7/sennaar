use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Result as FmtResult;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::cthulhu::extend_lifetime;


struct IdentInternal {
    original: String,
    renamed: RefCell<Option<String>>
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

impl Identifier {
    pub fn original(&self) -> &str {
        &self.0.original
    }

    pub fn value<'a>(&'a self) -> &'a str {
        self.renamed().unwrap_or(&self.0.original)
    }

    pub fn renamed<'a>(&'a self) -> Option<&'a str> {
        self.0.renamed
            .borrow()
            .as_ref()
            // SAFETY: As we only allow renaming one identifier once, the returned string is
            // guaranteed to be valid as long as the Rc<RenameState> is alive. And since that Rc is
            // held by the Identifier, it is guaranteed to be valid for the lifetime of the Identifier.
            .map(|s| unsafe { extend_lifetime(s.as_str()) })
    }

    pub fn try_rename(&self, new_name: &str) -> Result<(), String> {
        if new_name.contains(':') {
            return Err("Renamed identifiers cannot contain ':'".to_string());
        }

        let mut renamed = self.0.renamed.borrow_mut();
        if let Some(current) = renamed.as_ref() {
            if current == new_name {
                return Ok(());
            }
            return Err(format!(
                "Cannot rename identifier '{}' to '{}': already renamed to '{}'",
                self.0.original, new_name, current
            ));
        }
        renamed.replace(new_name.to_string());
        Ok(())
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
            .then(self_addr.cmp(&other_addr))
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
        let parts: Vec<&str> = s.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let ident = parts[0].interned();
            match ident.try_rename(parts[1]) {
                Ok(_) => Ok(ident),
                Err(e) => {
                    Err(DeserializeError::custom(format!(
                        "Failed to rename identifier '{}': {}",
                        ident.0.original, e
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
        assert!(!self.contains(':'), "Identifiers cannot contain ':'");

        IDENTIFIERS.with(|renames| {
            let mut renames = renames.borrow_mut();
            let internal = if let Some(state) = renames.get(self) {
                state.clone()
            } else {
                let state = Rc::new(IdentInternal {
                    original: self.to_string(),
                    renamed: RefCell::new(None),
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

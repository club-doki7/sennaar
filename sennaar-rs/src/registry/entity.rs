use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cpl::CExpr;
use crate::registry::{Metadata, Type};
use crate::Identifier;

pub enum Entities<'de> {
    Typedef(Typedef<'de>),
    FunctionTypedef(FunctionTypedef<'de>),
    Command(Command<'de>),
}

pub trait Entity<'de> : Sized + Eq + Ord + Serialize + Deserialize<'de> {
    fn entity_metadata(&self) -> &HashMap<String, Metadata>;
    fn entity_metadata_mut(&mut self) -> &mut HashMap<String, Metadata>;

    fn has_metadata(&self, key: &str) -> bool {
        self.entity_metadata().contains_key(key)
    }

    fn try_get_metadata(&self, key: &str) -> Option<&Metadata> {
        self.entity_metadata().get(key)
    }

    fn get_metadata(&self, key: &str) -> &Metadata {
        self.try_get_metadata(key).unwrap()
    }

    fn get_string_metadata(&self, key: &str) -> Option<&String> {
        self.try_get_metadata(key).and_then(|meta| {
            if let Metadata::String { value } = meta {
                Some(value)
            } else {
                panic!("expected string metadata for key '{}', found {:?}", key, meta);
            }
        })
    }

    fn get_kvs_metadata(&self, key: &str) -> Option<&HashMap<String, Metadata>> {
        self.try_get_metadata(key).and_then(|meta| {
            if let Metadata::KeyValues { kvs } = meta {
                Some(kvs)
            } else {
                panic!("expected key-values metadata for key '{}', found {:?}", key, meta);
            }
        })
    }

    fn put_metadata_kv(&mut self, key: impl ToString, value: Metadata) {
        self.entity_metadata_mut().insert(key.to_string(), value);
    }

    fn put_metadata(&mut self, key: impl AsRef<str> + ToString) {
        let metadata_mut = self.entity_metadata_mut();
        if !metadata_mut.contains_key(key.as_ref()) {
            metadata_mut.insert(key.to_string(), Metadata::None);
        }
    }

    fn put_metadata_string(&mut self, key: impl ToString, value: impl ToString) {
        self.put_metadata_kv(key, Metadata::String { value: value.to_string() });
    }

    fn put_metadata_kvs(&mut self, key: impl ToString, kvs: HashMap<String, Metadata>) {
        self.put_metadata_kv(key, Metadata::KeyValues { kvs });
    }
}

include!("../macross.rs");
include!("entity_macross.rs");

entity!{EntityBase,}

entity!{
    Typedef<'a>,
    target: Type<'a>,
}

ss_enum! {
    Bitwidth, Bit32, Bit64
}

entity!{
    Bitmask<'a>,
    bitwidth: Bitwidth,
    bitflags: Vec<Bitflag<'a>>
}

entity!{
    Bitflag<'a>,
    value: CExpr<'a>
}

entity!{
    Command<'a>,
    params: Vec<Param<'a>>,
    result: Type<'a>,
    success_codes: Vec<CExpr<'a>>,
    error_codes: Vec<CExpr<'a>>,
    alias_to: Option<Identifier>
}

impl<'a> Command<'a> {
    pub fn sanitize(&self) {
        for param in &self.params {
            param.sanitize();
        }
    }

    pub fn sanitize_fix(&mut self) {
        for param in &mut self.params {
            param.sanitize_fix();
        }
    }
}

entity!{
    Param<'a>,
    ty: Type<'a>,
    optional: bool,
    len: Option<CExpr<'a>>
}

impl<'a> Param<'a> {
    pub fn sanitize(&self) {
        if let Type::PointerType(ptr_type) = &self.ty {
            assert_eq!(ptr_type.nullable, self.optional);
        }
    }

    pub fn sanitize_fix(&mut self) {
        if let Type::PointerType(ptr_type) = &mut self.ty {
            ptr_type.nullable = self.optional;
        }
    }
}

entity!{
    Constant<'a>,
    ty: Type<'a>,
    expr: CExpr<'a>,
}

entity!{
    Enumeration<'a>,
    variants: Vec<EnumVariant<'a>>,
}

entity!{
    EnumVariant<'a>,
    value: CExpr<'a>
}

entity!{
    FunctionTypedef<'a>,
    params: Vec<Param<'a>>,
    result: Type<'a>,
    is_pointer: bool,
    is_native_api: bool
}

impl<'a> FunctionTypedef<'a> {
    pub fn sanitize(&self) {
        for param in &self.params {
            param.sanitize();
        }
    }

    pub fn sanitize_fix(&mut self) {
        for param in &mut self.params {
            param.sanitize_fix();
        }
    }
}

entity!{OpaqueTypedef,}

entity!{OpaqueHandleTypedef,}

entity!{
    Structure<'a>,
    members: Vec<Member<'a>>,
}

entity!{
    Member<'a>,
    ty: Type<'a>,
    bits: Option<usize>,
    init: Option<CExpr<'a>>,
    optional: bool,
    len: Option<CExpr<'a>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct Import {
    pub name: Identifier,
    pub version: Option<String>,
    pub depend: bool
}

impl PartialEq for Import {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Import {}

impl PartialOrd for Import {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Import {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

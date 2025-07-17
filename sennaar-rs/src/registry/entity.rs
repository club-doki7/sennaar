use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cpl::CExpr;
use crate::registry::{Metadata, Platform, Type};
use crate::Identifier;

pub trait Entity<'de> : Eq + Ord + Serialize + Deserialize<'de> {
    fn entity_name(&self) -> &Identifier;
    fn entity_metadata(&self) -> &HashMap<String, Metadata>;
    fn entity_metadata_mut(&mut self) -> &mut HashMap<String, Metadata>;
    fn entity_doc(&self) -> &[String];
    fn entity_doc_mut(&mut self) -> &mut Vec<String>;
    fn entity_platform(&self) -> Option<&Platform>;

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

include!("../macross.rs");
include!("entity_macross.rs");

entity!{
    Typedef,
    target: Identifier,
}

ss_enum! {
    Bitwidth, Bit32, Bit64
}

entity_a!{
    Bitmask,
    bitwidth: Bitwidth,
    bitflags: Vec<Bitflag<'a>>
}

entity_a!{
    Bitflag,
    value: CExpr<'a>
}

entity_a!{
    Command,
    params: Vec<Param<'a>>,
    result: Type<'a>,
    success_codes: Vec<CExpr<'a>>,
    error_codes: Vec<CExpr<'a>>,
    alias_to: Option<Identifier>
}

entity_a!{
    Param,
    ty: Type<'a>,
    optional: bool,
    len: Option<CExpr<'a>>,
    arg_len: Option<CExpr<'a>>
}

entity_a!{
    Constant,
    ty: Type<'a>,
    expr: CExpr<'a>,
}

entity_a!{
    Enumeration,
    variants: Vec<EnumVariant<'a>>,
}

entity_a!{
    EnumVariant,
    value: CExpr<'a>
}

entity_a!{
    FunctionTypedef,
    params: Vec<Param<'a>>,
    result: Type<'a>,
    is_pointer: bool,
    is_native_api: bool
}

entity!{OpaqueTypedef,}

entity!{OpaqueHandleTypedef,}

entity_a!{
    Structure,
    members: Vec<Member<'a>>,
}

entity_a!{
    Member,
    ty: Type<'a>,
    bits: usize,
    init: Option<CExpr<'a>>,
    optional: bool,
    len: Option<CExpr<'a>>,
    alt_len: Option<CExpr<'a>>,
}

entity_a!{
    Registry,
    aliases: HashMap<Identifier, Typedef>,
    bitmasks: HashMap<Identifier, Bitmask<'a>>,
    constants: HashMap<Identifier, Constant<'a>>,
    commands: HashMap<Identifier, Command<'a>>,
    enumerations: HashMap<Identifier, Enumeration<'a>>,
    function_typedefs: HashMap<Identifier, FunctionTypedef<'a>>,
    opaque_typedefs: HashMap<Identifier, OpaqueTypedef>,
    opaque_handle_typedefs: HashMap<Identifier, OpaqueHandleTypedef>,
    structures: HashMap<Identifier, Structure<'a>>,
    unions: HashMap<Identifier, Structure<'a>>
}

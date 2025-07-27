use std::collections::{BTreeSet, HashMap};

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

entity_a!{
    Typedef,
    target: Type<'a>,
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

entity_a!{
    Param,
    ty: Type<'a>,
    optional: bool,
    len: Option<CExpr<'a>>,
    arg_len: Option<CExpr<'a>>
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

entity_a!{
    Registry,
    imports: BTreeSet<Import>,
    aliases: HashMap<Identifier, Typedef<'a>>,
    bitmasks: HashMap<Identifier, Bitmask<'a>>,
    commands: HashMap<Identifier, Command<'a>>,
    constants: HashMap<Identifier, Constant<'a>>,
    enumerations: HashMap<Identifier, Enumeration<'a>>,
    function_typedefs: HashMap<Identifier, FunctionTypedef<'a>>,
    opaque_typedefs: HashMap<Identifier, OpaqueTypedef>,
    opaque_handle_typedefs: HashMap<Identifier, OpaqueHandleTypedef>,
    structs: HashMap<Identifier, Structure<'a>>,
    unions: HashMap<Identifier, Structure<'a>>
}

impl<'a> Registry<'a> {
    pub fn new(name: Identifier) -> Self {
        Self {
            name,
            metadata: HashMap::new(),
            doc: Vec::new(),
            platform: None,

            imports: BTreeSet::new(),
            aliases: HashMap::new(),
            bitmasks: HashMap::new(),
            constants: HashMap::new(),
            commands: HashMap::new(),
            enumerations: HashMap::new(),
            function_typedefs: HashMap::new(),
            opaque_typedefs: HashMap::new(),
            opaque_handle_typedefs: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new()
        }
    }

    pub fn sanitize(&self) {
        for command in self.commands.values() {
            command.sanitize();
        }

        for typedef in self.function_typedefs.values() {
            typedef.sanitize();
        }
    }

    pub fn sanitize_fix(&mut self) {
        for command in self.commands.values_mut() {
            command.sanitize_fix();
        }

        for typedef in self.function_typedefs.values_mut() {
            typedef.sanitize_fix();
        }
    }

    // TODO: Unlikly, but how to deal with colliding items?
    pub fn merge_with(&mut self, other: Self) {
        self.imports.extend(other.imports);
        self.aliases.extend(other.aliases);
        self.bitmasks.extend(other.bitmasks);
        self.constants.extend(other.constants);
        self.commands.extend(other.commands);
        self.enumerations.extend(other.enumerations);
        self.function_typedefs.extend(other.function_typedefs);
        self.opaque_typedefs.extend(other.opaque_typedefs);
        self.opaque_handle_typedefs.extend(other.opaque_handle_typedefs);
        self.structs.extend(other.structs);
        self.unions.extend(other.unions);

        for (key, value) in other.metadata {
            self.metadata.entry(key).or_insert(value);
        }

        self.doc.extend(other.doc);
    }
}

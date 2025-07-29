use std::collections::{BTreeSet, HashMap};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::Identifier;
use crate::registry::entity::*;

include!("registry_macross.rs");

registry!{RegistryBase,}

impl<'a> RegistryBase<'a> {
    pub fn merge_base_with(&mut self, other: RegistryBase<'a>) {
        // TODO: Unlikly, but how to deal with colliding items?
        self.metadefs.extend(other.metadefs);
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
    }
}

registry!{Registry, ext: serde_json::Value}

registry!{RegistryTE<EXT>, ext: EXT}

impl<'a> Registry<'a> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            imports: BTreeSet::new(),
            metadefs: HashMap::new(),

            aliases: HashMap::new(),
            bitmasks: HashMap::new(),
            constants: HashMap::new(),
            commands: HashMap::new(),
            enumerations: HashMap::new(),
            function_typedefs: HashMap::new(),
            opaque_typedefs: HashMap::new(),
            opaque_handle_typedefs: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            ext: serde_json::Value::Null
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

    pub fn as_base<'b>(&'b self) -> &'b RegistryBase<'a> {
        unsafe {
            &*(self as *const Registry<'a> as *const RegistryBase<'a>)
        }
    }

    pub fn as_base_mut<'b>(&'b mut self) -> &'b mut RegistryBase<'a> {
        unsafe {
            &mut *(self as *mut Registry<'a> as *mut RegistryBase<'a>)
        }
    }
}

impl<'a, EXT: 'a + Default> RegistryTE<'a, EXT> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            imports: BTreeSet::new(),
            metadefs: HashMap::new(),

            aliases: HashMap::new(),
            bitmasks: HashMap::new(),
            constants: HashMap::new(),
            commands: HashMap::new(),
            enumerations: HashMap::new(),
            function_typedefs: HashMap::new(),
            opaque_typedefs: HashMap::new(),
            opaque_handle_typedefs: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            ext: Default::default()
        }
    }
}

impl<'a, EXT: 'a> RegistryTE<'a, EXT> {
    pub fn new_with_ext(name: String, ext: EXT) -> Self {
        Self {
            name,
            imports: BTreeSet::new(),
            metadefs: HashMap::new(),

            aliases: HashMap::new(),
            bitmasks: HashMap::new(),
            constants: HashMap::new(),
            commands: HashMap::new(),
            enumerations: HashMap::new(),
            function_typedefs: HashMap::new(),
            opaque_typedefs: HashMap::new(),
            opaque_handle_typedefs: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            ext
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

    pub fn as_base<'b>(&'b self) -> &'b RegistryBase<'a> {
        unsafe {
            &*(self as *const RegistryTE<'a, EXT> as *const RegistryBase<'a>)
        }
    }

    pub fn as_base_mut<'b>(&'b mut self) -> &'b mut RegistryBase<'a> {
        unsafe {
            &mut *(self as *mut RegistryTE<'a, EXT> as *mut RegistryBase<'a>)
        }
    }
}

impl<'a, 'de, EXT: 'a + DeserializeOwned> TryFrom<Registry<'a>> for RegistryTE<'a, EXT> {
    type Error = serde_json::Error;

    fn try_from(registry: Registry<'a>) -> Result<RegistryTE<'a, EXT>, serde_json::Error> {
        let ext = serde_json::from_value::<EXT>(registry.ext)?;

        Ok(Self {
            name: registry.name,
            imports: registry.imports,
            metadefs: registry.metadefs,

            aliases: registry.aliases,
            bitmasks: registry.bitmasks,
            constants: registry.constants,
            commands: registry.commands,
            enumerations: registry.enumerations,
            function_typedefs: registry.function_typedefs,
            opaque_typedefs: registry.opaque_typedefs,
            opaque_handle_typedefs: registry.opaque_handle_typedefs,
            structs: registry.structs,
            unions: registry.unions,

            ext
        })
    }
}

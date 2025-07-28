macro_rules! entity {
    ($name:ident $(<$lifetime:lifetime>)?, $($field:ident: $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[derive(JsonSchema)]
        #[serde(rename_all = "camelCase")]
        #[repr(C)]
        pub struct $name$(<$lifetime>)? {
            pub name: crate::Identifier,
            pub metadata: HashMap<String, Metadata>,
            pub doc: Vec<String>,
            pub platform: Option<crate::registry::Platform>,
            $(pub $field: $type),*
        }

        impl<'de $(,$lifetime)?> Entity<'de> for $name$(<$lifetime>)? {
            fn entity_metadata(&self) -> &HashMap<String, Metadata> {
                &self.metadata
            }

            fn entity_metadata_mut(&mut self) -> &mut HashMap<String, Metadata> {
                &mut self.metadata
            }
        }

        impl$(<$lifetime>)? PartialEq for $name$(<$lifetime>)? {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }

        impl$(<$lifetime>)? Eq for $name$(<$lifetime>)? {}

        impl$(<$lifetime>)? PartialOrd for $name$(<$lifetime>)? {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl$(<$lifetime>)? Ord for $name$(<$lifetime>)? {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.name.cmp(&other.name)
            }
        }
    };
}

macro_rules! registry {
    ($name:ident$(<$generic:ident>)?, $($field:ident: $type:ty),*$(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[derive(JsonSchema)]
        #[serde(rename_all = "camelCase")]
        #[repr(C)]
        pub struct $name<'a, $($generic: 'a,)?> {
            pub name: String,
            pub imports: BTreeSet<Import>,

            pub aliases: HashMap<Identifier, Typedef<'a>>,
            pub bitmasks: HashMap<Identifier, Bitmask<'a>>,
            pub constants: HashMap<Identifier, Constant<'a>>,
            pub commands: HashMap<Identifier, Command<'a>>,
            pub enumerations: HashMap<Identifier, Enumeration<'a>>,
            pub function_typedefs: HashMap<Identifier, FunctionTypedef<'a>>,
            pub opaque_typedefs: HashMap<Identifier, OpaqueTypedef>,
            pub opaque_handle_typedefs: HashMap<Identifier, OpaqueHandleTypedef>,
            pub structs: HashMap<Identifier, Structure<'a>>,
            pub unions: HashMap<Identifier, Structure<'a>>,

            $(pub $field: $type),*
        }
    }
}

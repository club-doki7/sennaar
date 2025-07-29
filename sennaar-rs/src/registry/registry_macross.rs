macro_rules! registry {
    ($name:ident$(<$generic:ident>)?, $($field:ident: $type:ty),*$(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[derive(JsonSchema)]
        #[serde(rename_all = "camelCase")]
        #[repr(C)]
        pub struct $name<'a, $($generic: 'a,)?> {
            pub name: String,
            pub metadefs: HashMap<String, String>,
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

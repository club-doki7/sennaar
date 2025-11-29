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

        impl $(<$lifetime>)? $name$(<$lifetime>)? {
            pub fn new(name: crate::Identifier, $($field: $type),*) -> $name$(<$lifetime>)? {
                $name {
                    name,
                    metadata: HashMap::new(),
                    doc: Vec::new(),
                    platform: Option::None,
                    $($field),*
                }
            }
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

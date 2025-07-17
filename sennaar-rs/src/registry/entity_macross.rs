macro_rules! entity {
    ($name:ident, $($field:ident: $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub name: crate::Identifier,
            pub metadata: HashMap<String, Metadata>,
            pub doc: Vec<String>,
            pub platform: Option<crate::registry::Platform>,
            $(pub $field: $type),*
        }

        impl<'de> Entity<'de> for $name {
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

            fn entity_platform(&self) -> Option<&crate::registry::Platform> {
                self.platform.as_ref()
            }
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
    };
}

macro_rules! entity_a {
    ($name:ident, $($field:ident: $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $name<'a> {
            pub name: crate::Identifier,
            pub metadata: HashMap<String, Metadata>,
            pub doc: Vec<String>,
            pub platform: Option<crate::registry::Platform>,
            $(pub $field: $type),*
        }

        impl<'de> Entity<'de> for $name<'_> {
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

            fn entity_platform(&self) -> Option<&crate::registry::Platform> {
                self.platform.as_ref()
            }
        }

        impl PartialEq for $name<'_> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }

        impl Eq for $name<'_> {}

        impl PartialOrd for $name<'_> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name<'_> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.name.cmp(&other.name)
            }
        }
    }
}
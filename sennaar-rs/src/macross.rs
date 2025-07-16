macro_rules! ss_enum {
    ($name:ident, $($variant:ident),+) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(match self {
                    $(Self::$variant => stringify!($variant)),+
                })
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s: &str = serde::Deserialize::deserialize(deserializer)?;
                match s {
                    $(stringify!($variant) => Ok(Self::$variant)),+,
                    _ => Err(serde::de::Error::custom(format!("Unknown {} variant: {}", stringify!($name), s))),
                }
            }
        }
    };
}

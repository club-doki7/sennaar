#[allow(unused_macros)]
macro_rules! ss_enum {
    ($name:ident, $($variant:ident),+) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        pub enum $name {
            $($variant),+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$variant => stringify!($variant)),+
                })
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($variant) => Ok(Self::$variant)),+,
                    _ => Err(format!("Unknown variant: {}", s)),
                }
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! ss_enum_wcustom {
    ($name:ident, $($variant:ident),+) => {
        #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum $name {
            Custom(String),
            $($variant),+
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(match self {
                    Self::Custom(s) => s,
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
                    _ => Ok(Self::Custom(s.to_string())),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    Self::Custom(s) => s,
                    $(Self::$variant => stringify!($variant)),+
                })
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($variant) => Ok(Self::$variant)),+,
                    _ => Ok(Self::Custom(s.to_string())),
                }
            }
        }
    }
}

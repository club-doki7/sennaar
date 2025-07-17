use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

include!("../macross.rs");

ss_enum_wcustom! {
    Arch,
    I386, X86_64, AArch64, RiscV64
}

ss_enum! {
    Endian, Little, Big
}

ss_enum_wcustom! {
    OS,
    Windows, Linux, MacOS
}

ss_enum_wcustom! {
    LibC,
    MUSL, GLIBC, MSFT
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct Platform {
    pub arch: Option<Arch>,
    pub endian: Option<Endian>,
    pub os: Option<OS>,
    pub libc: Option<LibC>,
    pub custom: Option<String>,
}

pub const UNKNOWN_ARCH: &'static str = "UnknownArch";
pub const UNKNOWN_ENDIAN: &'static str = "UnknownEndian";
pub const UNKNOWN_OS: &'static str = "UnknownOS";
pub const UNKNOWN_LIBC: &'static str = "UnknownLibC";

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(arch) = &self.arch {
            write!(f, "{arch}")?;
        } else {
            write!(f, "{UNKNOWN_ARCH}")?;
        }
        write!(f, "-")?;

        if let Some(endian) = &self.endian {
            write!(f, "{endian}")?;
        } else {
            write!(f, "{UNKNOWN_ENDIAN}")?;
        }

        write!(f, "-")?;
        if let Some(os) = &self.os {
            write!(f, "{os}")?;
        } else {
            write!(f, "{UNKNOWN_OS}")?;
        }

        write!(f, "-")?;
        if let Some(libc) = &self.libc {
            write!(f, "{libc}")?;
        } else {
            write!(f, "{UNKNOWN_LIBC}")?;
        }

        if let Some(custom) = &self.custom {
            write!(f, "-[{}]", custom)?;
        }

        Ok(())
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(5, '-').collect();
        if parts.len() < 4 {
            return Err("Platform string must have at least 4 parts".to_string());
        }

        let arch = if parts[0] == UNKNOWN_ARCH {
            None
        } else {
            Some(parts[0].parse::<Arch>().unwrap())
        };

        let endian = if parts[1] == UNKNOWN_ENDIAN {
            None
        } else {
            Some(parts[1].parse::<Endian>()?)
        };

        let os = if parts[2] == UNKNOWN_OS {
            None
        } else {
            Some(parts[2].parse::<OS>().unwrap())
        };

        let libc = if parts[3] == UNKNOWN_LIBC {
            None
        } else {
            Some(parts[3].parse::<LibC>().unwrap())
        };

        let custom = if parts.len() == 5 {
            Some(parts[4].trim_start_matches('[').trim_end_matches(']').to_string())
        } else {
            None
        };

        Ok(Platform { arch, endian, os, libc, custom })
    }
}

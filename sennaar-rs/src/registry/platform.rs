#![allow(non_camel_case_types)]

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


include!("../macross.rs");

ss_enum_wcustom! {
    Arch,
    i386, amd64, aarch64, riscv64
}

ss_enum! {
    Endian, little, big
}

ss_enum_wcustom! {
    OS,
    windows, linux, macos, freebsd
}

ss_enum_wcustom! {
    LibC,
    msft, musl, glibc
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[derive(JsonSchema)]
#[serde(tag = "$kind")]
pub enum PlatformSpecifierState<T> {
    Exact { value: T },
    Other,
    Any
}

impl<T: Display> PlatformSpecifierState<T> {
    fn write_with_other_and_any(
        &self,
        f: &mut Formatter<'_>,
        other: &str,
        any: &str,
    ) -> FmtResult {
        match self {
            PlatformSpecifierState::Exact { value } => write!(f, "{value}"),
            PlatformSpecifierState::Other => write!(f, "{other}"),
            PlatformSpecifierState::Any => write!(f, "{any}"),
        }
    }
}

impl<T: FromStr> PlatformSpecifierState<T> {
    pub fn parse_with_other_and_any(
        s: &str,
        other: &str,
        any: &str,
    ) -> Result<Self, T::Err> {
        if s == other {
            Ok(PlatformSpecifierState::Other)
        } else if s == any {
            Ok(PlatformSpecifierState::Any)
        } else {
            Ok(PlatformSpecifierState::Exact { value: s.parse()? })
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct Platform {
    pub arch: PlatformSpecifierState<Arch>,
    pub endian: Option<Endian>,
    pub os: PlatformSpecifierState<OS>,
    pub libc: PlatformSpecifierState<LibC>,
    pub custom: PlatformSpecifierState<String>,
}

pub const ANY_ARCH: &'static str = "any_arch";
pub const ANY_ENDIAN: &'static str = "any_endian";
pub const ANY_OS: &'static str = "any_os";
pub const ANY_LIBC: &'static str = "any_libc";

pub const OTHER_ARCH: &'static str = "other_arch";
pub const OTHER_ENDIAN: &'static str = "other_endian";
pub const OTHER_OS: &'static str = "other_os";
pub const OTHER_LIBC: &'static str = "other_libc";

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.arch.write_with_other_and_any(f, OTHER_ARCH, ANY_ARCH)?;
        write!(f, "-")?;

        if let Some(endian) = &self.endian {
            write!(f, "{endian}-")?;
        } else {
            write!(f, "{ANY_ENDIAN}-")?;
        }

        self.os.write_with_other_and_any(f, OTHER_OS, ANY_OS)?;
        write!(f, "-")?;

        self.libc.write_with_other_and_any(f, OTHER_LIBC, ANY_LIBC)?;

        if let PlatformSpecifierState::Exact { value } = &self.custom {
            write!(f, "-[{value}]")
        } else {
            write!(f, "-[]")
        }?;

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

        let arch = PlatformSpecifierState::parse_with_other_and_any(parts[0], OTHER_ARCH, ANY_ARCH).unwrap();
        let endian = if parts[1] == ANY_ENDIAN {
            None
        } else {
            Some(parts[1].parse()?)
        };

        let os = PlatformSpecifierState::parse_with_other_and_any(parts[2], OTHER_OS, ANY_OS).unwrap();
        let libc = PlatformSpecifierState::parse_with_other_and_any(parts[3], OTHER_LIBC, ANY_LIBC).unwrap();

        let custom = if parts.len() > 4 && !parts[4].is_empty() {
            PlatformSpecifierState::Exact { value: parts[4].to_string() }
        } else {
            PlatformSpecifierState::Any
        };

        Ok(Platform {
            arch,
            endian,
            os,
            libc,
            custom,
        })
    }
}

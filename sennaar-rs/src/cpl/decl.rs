use std::fmt::Display;

use either::Either;

use crate::{
    Identifier,
    cpl::{CParam, CType},
};

// TODO: remove unnecessary Box
#[derive(Debug)]
pub enum CDecl {
    Typedef(Box<CTypedefDecl>),
    Fn(Box<CFnDecl>),
    Struct(Box<CStructDecl>),
    Union(Box<CStructDecl>),
    Enum(Box<CEnumDecl>),
    Var(Box<CVarDecl>),
}

#[derive(Debug)]
pub struct CTypedefDecl {
    pub name: Identifier,
    pub underlying: CType,
}

#[derive(Debug)]
pub struct CFnDecl {
    pub name: Identifier,
    pub ret: Box<CType>,
    pub parameters: Vec<CParam>,
}

pub type RecordName = Either<Identifier, String>;

#[derive(Debug)]
pub struct CStructDecl {
    /// either a named struct or a unnamed struct with its USR
    pub name: RecordName,
    pub fields: Vec<CFieldDecl>,
    /// whether this decl is a definition, false implies fields and subrecords are empty
    pub is_definition: bool,
    // TODO: after naming, all [subrecords] will store the identifier of those struct instead of USR
    pub subrecords: Vec<RecordName>,
}

#[derive(Debug)]
pub struct CFieldDecl {
    pub name: Identifier,
    pub ty: CType,
}

#[derive(Debug)]
pub struct CEnumDecl {
    pub name: Identifier,
    pub ty: CType,
    pub members: Vec<CEnumConstantDecl>,
}

/// @param explicit whether the value of this decl is explicit
#[derive(Debug)]
pub struct CEnumConstantDecl {
    pub name: Identifier,
    pub explicit: bool,
    pub value: u64,
}

#[derive(Debug)]
pub struct CVarDecl {
    pub name: Identifier, 
    pub ty: CType,
}

impl CDecl {
    pub fn get_record_decl(&self) -> Option<&CStructDecl> {
        match &self {
            CDecl::Struct(decl) => Some(decl),
            CDecl::Union(decl) => Some(decl),
            _ => None
        }
    }

    pub fn move_record_decl(self) -> Option<CStructDecl> {
        match self {
            CDecl::Struct(record) | CDecl::Union(record) => Some(*record),
            CDecl::Typedef(_) 
            | CDecl::Fn(_) 
            | CDecl::Enum(_)
            | CDecl::Var(_) => None,
        }
    }

    pub fn name(&self) -> RecordName {
        match &self {
            CDecl::Typedef(decl) => Either::Left(decl.name.clone()),
            CDecl::Fn(decl) => Either::Left(decl.name.clone()),
            CDecl::Struct(decl) => decl.name.clone(),
            CDecl::Union(decl) => decl.name.clone(),
            CDecl::Enum(decl) => Either::Left(decl.name.clone()),
            CDecl::Var(decl) => Either::Left(decl.name.clone()),
        }
    }

    pub fn fmt_struct_like(f: &mut std::fmt::Formatter<'_>, keyword: &'static str, decl: &CStructDecl) -> std::fmt::Result {
        write!(f, "{} ", keyword)?;

        match &decl.name {
            Either::Left(name) => write!(f, "{}", name)?,
            Either::Right(usr) => write!(f, "/* USR: {} */", usr)?,
        }

        if decl.is_definition {
            write!(f, " {{")?;

            decl.fields
                .iter()
                .try_for_each(|field| write!(f, " {} {};", field.ty, field.name))?;

            decl.subrecords
                .iter()
                .try_for_each(|subdecl| {
                    write!(f, " <subdecl ")?;
                    match subdecl {
                        Either::Left(ident) => write!(f, "{}", ident)?,
                        Either::Right(usr) => write!(f, "USR: {}", usr)?,
                    }

                    write!(f, ">;")
                })?;

            write!(f, " }}")?;
        } 

        write!(f, ";")
    }
}

impl Display for CDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CDecl::Typedef(decl) => {
                write!(f, "typedef {} {};", decl.underlying, decl.name)
            }
            CDecl::Fn(decl) => {
                CType::fmt_fun(f, &decl.ret, &decl.parameters, Some(&decl.name), false, false)?;

                write!(f, ";")
            }
            CDecl::Struct(decl) => CDecl::fmt_struct_like(f, "struct", decl),
            CDecl::Union(decl) => CDecl::fmt_struct_like(f, "union", decl),
            CDecl::Enum(decl) => {
                write!(f, "enum {} {{ ", decl.name)?;

                for (idx, member) in decl.members.iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }

                    if member.explicit {
                        write!(f, "{} = {}", member.name, member.value)?;
                    } else {
                        write!(f, "{}", member.name)?;
                    }
                }

                write!(f, " }};")
            },
            CDecl::Var(decl) => {
                write!(f, "{} {};", decl.ty, decl.name)
            }
        }
    }
}

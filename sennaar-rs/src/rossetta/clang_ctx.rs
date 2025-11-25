use std::collections::HashMap;

use crate::cpl::CStructDecl;

pub struct ClangCtx {
    unnamed_structs: HashMap<String, CStructDecl>,
}

impl ClangCtx {
    pub fn new() -> ClangCtx {
        ClangCtx {
            unnamed_structs: HashMap::new()
        }
    }

    pub fn find_unnamed_struct(&self, usr: String) -> Option<&CStructDecl> {
        self.unnamed_structs.get(&usr)
    }

    pub fn insert_unnamed_struct(&mut self, usr: String, decl: CStructDecl) -> Option<CStructDecl> {
        self.unnamed_structs.insert(usr, decl)
    }
}

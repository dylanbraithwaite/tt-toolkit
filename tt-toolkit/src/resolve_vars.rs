use std::fmt::Display;

use cons_list::ConsList;

pub type NameContext = ConsList<String>;

#[derive(Debug)]
pub enum ResolveVarsError {
    UnboundVariable(String),
}

impl Display for ResolveVarsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveVarsError::UnboundVariable(var) => {
                write!(f, "Unbound variable name: {var}")
            }
        }
    }
}

impl std::error::Error for ResolveVarsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub trait ResolveVars: Sized {
    fn resolve_vars(
        &self,
        context: &NameContext,
    ) -> Result<Self, ResolveVarsError>;
}

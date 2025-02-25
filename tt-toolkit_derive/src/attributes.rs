use crate::utils::attributes::HasAttributes;

pub const METADATA_ATTR: &str = "metadata";
pub const VAR_NAME_ATTR: &str = "var_name";
pub const BINDING_ATTR: &str = "binding";
pub const BINDING_NAME_ATTR: &str = "binding_name";
pub const DEBRUIJN_VAR_ATTR: &str = "var_index";
pub const VAR_WRAPPER_ATTR: &str = "variable";
pub const SUBST_TYPES_ATTR: &str = "subst_types";

// Evaluate
pub const CONTEXT_TYPE_ATTR: &str = "context_type";
pub const EVAL_TARGET_ATTR: &str = "eval_target";
pub const EVAL_ERROR_ATTR: &str = "eval_error_type";
pub const EVAL_FUNC_ATTR: &str = "evaluate_with";
pub const EVAL_PATTERN_ATTR: &str = "evaluate_pattern";
pub const EVAL_UNWRAP_ATTR: &str = "evaluate_unwrap_variant";

pub trait IsMetadata {
    fn is_metadata(&self) -> bool;
}

impl<T: HasAttributes> IsMetadata for T {
    fn is_metadata(&self) -> bool {
        self.has_attribute(METADATA_ATTR)
            || self.has_attribute(VAR_NAME_ATTR)
            || self.has_attribute(BINDING_NAME_ATTR)
    }
}

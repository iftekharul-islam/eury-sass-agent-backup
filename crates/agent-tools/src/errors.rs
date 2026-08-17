use crate::registry::ToolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaxonomyError {
    #[error("EURY_TOOL_NOT_FOUND: {0}")]
    NotFound(String),
    #[error("EURY_TOOL_EDIT_AMBIGUOUS: {0}")]
    EditAmbiguous(String),
    #[error("EURY_TOOL_EXECUTION_FAILED: {0}")]
    ExecutionFailed(String),
    #[error("EURY_TOOL_OUTPUT_TRUNCATED: {0}")]
    OutputTruncated(String),
}

impl From<TaxonomyError> for ToolError {
    fn from(err: TaxonomyError) -> Self {
        ToolError::Execution(err.to_string())
    }
}

mod commands;
mod emitter;

use thiserror::Error;

pub use emitter::Emitter;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("codegen failed: {0}")]
    Message(String),
}

pub fn codegen(script: &ironhotkey_parser::ast::Script) -> Result<String, CodegenError> {
    Emitter::new().emit(script)
}

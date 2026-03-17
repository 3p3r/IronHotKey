pub mod ast;
mod transform;

use thiserror::Error;
use tree_sitter::Parser;

pub use transform::Transformer;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("tree-sitter parse failed")]
    TreeSitter,
    #[error("language setup failed")]
    Language,
}

pub fn parse(source: &str) -> Result<ast::Script, ParseError> {
    let mut parser = Parser::new();
    let language = ironhotkey_grammar::language();
    parser
        .set_language(&language)
        .map_err(|_| ParseError::Language)?;

    let tree = parser.parse(source, None).ok_or(ParseError::TreeSitter)?;
    let root = tree.root_node();
    Ok(Transformer::new(source, root).transform())
}

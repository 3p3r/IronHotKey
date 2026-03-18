use crate::CodegenError;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_transforms_typescript::strip_type;
use swc_ecma_visit::VisitMutWith;

pub fn transpile(ts_code: &str) -> Result<String, CodegenError> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(
        FileName::Custom("generated.ts".into()).into(),
        ts_code.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax::default()),
        EsVersion::Es2022,
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let mut module = parser
        .parse_module()
        .map_err(|error| CodegenError::Message(format!("typescript parse failed: {error:?}")))?;

    module.visit_mut_with(&mut strip_type());

    let mut output = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: source_map.clone(),
            comments: None,
            wr: JsWriter::new(source_map, "\n", &mut output, None),
        };
        emitter
            .emit_module(&module)
            .map_err(|error| CodegenError::Message(format!("javascript emit failed: {error}")))?;
    }

    String::from_utf8(output)
        .map_err(|error| CodegenError::Message(format!("javascript utf8 decode failed: {error}")))
}

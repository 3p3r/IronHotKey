#[test]
fn parse_codegen_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/hello.ahk");

    let script = ironhotkey_parser::parse(source).expect("parser should work");
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());

    let js = ironhotkey_codegen::codegen(&script).expect("codegen should work");
    assert!(js.contains("ahk.gui.MsgBox"));
    assert!(js.contains("ahk.env.set"));

    ironhotkey_runtime::run(&js).expect("runtime should execute JS");
}

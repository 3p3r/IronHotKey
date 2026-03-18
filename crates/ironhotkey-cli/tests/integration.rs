#[test]
fn parse_codegen_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/hello.ahk");

    let script = ironhotkey_parser::parse(source).expect("parser should work");
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());

    let ts = ironhotkey_codegen::codegen(&script).expect("codegen should work");
    assert!(ts.contains("interface AhkRuntime"));
    assert!(ts.contains("declare const ahk: AhkRuntime"));
    assert!(!ts.contains("any"));
    assert!(ts.contains("ahk.gui.MsgBox"));
    assert!(ts.contains("ahk.env.set"));

    let js = ironhotkey_codegen::transpile(&ts).expect("transpile should work");
    ironhotkey_runtime::run(&js).expect("runtime should execute JS");
}

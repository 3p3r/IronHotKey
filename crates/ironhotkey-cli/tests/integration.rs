fn compile_fixture(source: &str) -> (ironhotkey_parser::ast::Script, String, String) {
    let script = ironhotkey_parser::parse(source).expect("parser should work");
    let ts = ironhotkey_codegen::codegen(&script).expect("codegen should work");
    let js = ironhotkey_codegen::transpile(&ts).expect("transpile should work");
    (script, ts, js)
}

#[test]
fn parse_codegen_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/hello.ahk");

    let (script, ts, js) = compile_fixture(source);
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());

    assert!(ts.contains("interface AhkRuntime"));
    assert!(ts.contains("declare const ahk: AhkRuntime"));
    assert!(!ts.contains("any"));
    assert!(ts.contains("ahk.gui.MsgBox"));
    assert!(ts.contains("ahk.env.set"));

    ironhotkey_runtime::run(&js).expect("runtime should execute JS");
}

#[test]
fn maths_function_codegen_and_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/maths.ahk");

    let (script, ts, js) = compile_fixture(source);
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());
    assert_eq!(script.auto_exec.len(), 27);

    for expected in [
        "ahk.maths.Abs(-1);",
        "ahk.maths.ACos(0.2);",
        "ahk.maths.ASin((-2.1 - 4));",
        "ahk.maths.Asc(\"A\");",
        "ahk.maths.ATan(1.2);",
        "ahk.maths.Ceil(-1.2);",
        "ahk.maths.Chr(65);",
        "ahk.maths.Cos(1);",
        "ahk.maths.Exp(42);",
        "ahk.maths.Floor(3.14159);",
        "ahk.maths.Format(\"{:010}\", \"00123\");",
        "ahk.maths.FormatTime(\"20240317010203\", \"yyyyMMddHHmmss\");",
        "ahk.maths.Ln(123);",
        "ahk.maths.Log(123);",
        "ahk.maths.Math(\"1.0e4+-2.1E-4\");",
        "ahk.maths.Max(2.11, -2, 0, 123);",
        "ahk.maths.Min(2.11, -2, 0, -1);",
        "ahk.maths.Mod(7.5, 2);",
        "ahk.maths.NumGet(\"ptr\", 16, \"UInt\");",
        "ahk.maths.NumPut(-1, \"ptr\", 123, \"Int64\");",
        "ahk.maths.Ord(\"Z\");",
        "ahk.maths.Random(-1, 1);",
        "ahk.maths.Round(3.14, 1);",
        "ahk.maths.Round(345, -2);",
        "ahk.maths.Sin(1.2);",
        "ahk.maths.Sqrt(9);",
        "ahk.maths.Tan(1.2);",
    ] {
        assert!(ts.contains(expected), "missing TS snippet: {expected}");
        assert!(
            js.contains(expected.trim_end_matches(';')),
            "missing JS snippet: {expected}"
        );
    }

    for source_number_syntax in [
        "00123", "-1", "0x7B", "0x007B", "-0x1", "3.14159", "42.0", "1.0e4", "-2.1E-4",
    ] {
        assert!(
            source.contains(source_number_syntax),
            "missing documented number syntax in fixture: {source_number_syntax}"
        );
    }

    ironhotkey_runtime::run(&js).expect("runtime should execute typed maths calls");
}

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

#[test]
fn string_function_codegen_and_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/string.ahk");

    let (script, ts, js) = compile_fixture(source);
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());
    assert_eq!(script.auto_exec.len(), 47);

    for expected in [
        "ahk.string.InStr(\"Hello World\", \"o\");",
        "ahk.string.StrLen(\"Hello\");",
        "ahk.string.StrReplace(\"Hello World\", \"World\", \"AHK\");",
        "ahk.string.SubStr(\"Hello World\", 1, 5);",
        "ahk.string.Trim(\"  Hello  \");",
        "ahk.string.LTrim(\"  Hello\");",
        "ahk.string.RTrim(\"Hello  \");",
        "ahk.string.RegExMatch(\"Hello123World\", \"\\\\\\\\d+\");",
        "ahk.string.RegExReplace(\"test123test\", \"\\\\\\\\d+\", \"X\");",
        "ahk.string.StringLen(\"Hello\");",
        "ahk.string.StringUpper(\"hello\");",
        "ahk.string.StringLower(\"HELLO\");",
        "ahk.string.StringTrimLeft(\"Hello World\", 6);",
        "ahk.string.StringTrimRight(\"Hello World\", 5);",
        "ahk.string.StringLeft(\"Hello\", 2);",
        "ahk.string.StringRight(\"Hello\", 3);",
        "ahk.string.StringMid(\"Hello World\", 7, 5);",
        "ahk.string.StringGetPos(\"Hello World\", \"World\");",
        "ahk.string.StringReplace(\"Hello World\", \"World\", \"AHK\");",
    ] {
        assert!(ts.contains(expected), "missing TS snippet: {expected}");
        assert!(
            js.contains(expected.trim_end_matches(';')),
            "missing JS snippet: {expected}"
        );
    }

    ironhotkey_runtime::run(&js).expect("runtime should execute typed string calls");
}

#[test]
fn disk_function_codegen_and_runtime_pipeline() {
    let source = include_str!("../../../tests/fixtures/disk.ahk");

    let (script, ts, js) = compile_fixture(source);
    assert!(!script.auto_exec.is_empty() || !script.directives.is_empty());

    for expected in [
        "ahk.disk.DriveGet(\"\", \"List\");",
        "ahk.disk.DriveSpaceFree(\"\", \"/\");",
        "ahk.disk.FileAppend(\"hello\", \"/tmp/ironhotkey_disk_fixture.txt\");",
        "ahk.disk.FileRead(\"/tmp/ironhotkey_disk_fixture.txt\");",
        "ahk.disk.FileReadLine(\"/tmp/ironhotkey_disk_fixture.txt\", 1);",
        "ahk.disk.FileExist(\"/tmp/ironhotkey_disk_fixture.txt\");",
        "ahk.disk.FileGetAttrib(\"/tmp/ironhotkey_disk_fixture.txt\");",
        "ahk.disk.FileGetSize(\"/tmp/ironhotkey_disk_fixture.txt\", \"B\");",
        "ahk.disk.FileGetTime(\"/tmp/ironhotkey_disk_fixture.txt\", \"M\");",
        "ahk.disk.SplitPath(\"/tmp/ironhotkey_disk_fixture.txt\");",
        "ahk.disk.LoopFile(\"/tmp/*\", \"F\");",
        "ahk.disk.LoopReadFile(\"/tmp/ironhotkey_disk_fixture.txt\");",
    ] {
        assert!(ts.contains(expected), "missing TS snippet: {expected}");
        assert!(
            js.contains(expected.trim_end_matches(';')),
            "missing JS snippet: {expected}"
        );
    }

    ironhotkey_runtime::run(&js).expect("runtime should execute typed disk calls");
}

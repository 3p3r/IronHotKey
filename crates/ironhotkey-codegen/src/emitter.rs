use crate::commands;
use crate::CodegenError;
use ironhotkey_parser::ast::*;
use ironhotkey_runtime::modules;

pub struct Emitter;

impl Emitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, script: &Script) -> Result<String, CodegenError> {
        let mut out = String::new();
        out.push_str("'use strict';\n");
        out.push_str("ahk.platform.init();\n");

        for directive in &script.directives {
            let args = directive
                .args
                .iter()
                .map(|arg| format!("\"{}\"", js_escape(arg)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "ahk.directives[\"{}\"]({});\n",
                js_escape(&directive.name),
                args
            ));
        }

        for label in &script.labels {
            out.push_str(&format!(
                "ahk.flow.registerLabel(\"{}\", () => {{\n",
                js_escape(&label.name)
            ));
            for statement in &label.body {
                self.emit_statement(statement, 1, &mut out)?;
            }
            out.push_str("});\n");
        }

        for function in &script.functions {
            out.push_str(&format!(
                "function {}({}): AhkResult {{\n",
                function.name,
                self.emit_params(&function.params)
            ));
            out.push_str("  ahk.env.pushScope();\n");
            for statement in &function.body {
                self.emit_statement(statement, 1, &mut out)?;
            }
            out.push_str("  ahk.env.popScope();\n");
            out.push_str("}\n");
            out.push_str(&format!(
                "ahk.flow.registerFunction(\"{}\", {});\n",
                js_escape(&function.name),
                function.name
            ));
        }

        for class in &script.classes {
            self.emit_class(class, &mut out)?;
        }

        for hotkey in &script.hotkeys {
            let mods = hotkey
                .modifiers
                .iter()
                .map(|m| format!("\"{}\"", self.modifier_name(m)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "ahk.mnk.registerHotkey([{}], \"{}\", () => {{\n",
                mods,
                js_escape(&hotkey.key)
            ));
            for statement in &hotkey.body {
                self.emit_statement(statement, 1, &mut out)?;
            }
            out.push_str("});\n");
        }

        for hotstring in &script.hotstrings {
            match &hotstring.replacement {
                HotstringAction::Text(text) => {
                    out.push_str(&format!(
                        "ahk.mnk.registerHotstring(\"{}\", \"{}\", \"{}\");\n",
                        js_escape(&hotstring.trigger),
                        js_escape(&hotstring.options),
                        js_escape(text)
                    ));
                }
                HotstringAction::Command(statements) => {
                    out.push_str(&format!(
                        "ahk.mnk.registerHotstring(\"{}\", \"{}\", () => {{\n",
                        js_escape(&hotstring.trigger),
                        js_escape(&hotstring.options)
                    ));
                    for statement in statements {
                        self.emit_statement(statement, 1, &mut out)?;
                    }
                    out.push_str("});\n");
                }
            }
        }

        for statement in &script.auto_exec {
            self.emit_statement(statement, 0, &mut out)?;
        }

        out.push_str("\n// --- AHK Runtime Type Definitions ---\n");
        self.emit_type_definitions(&mut out);

        Ok(out)
    }

    fn emit_statement(
        &self,
        statement: &Statement,
        indent: usize,
        out: &mut String,
    ) -> Result<(), CodegenError> {
        let pad = "  ".repeat(indent);
        match statement {
            Statement::Assignment { target, op, value } => {
                let name = self.emit_target_name(target);
                if matches!(op, AssignOp::Legacy) {
                    out.push_str(&format!(
                        "{}ahk.env.set(\"{}\", \"{}\");\n",
                        pad,
                        js_escape(&name),
                        js_escape(&self.emit_expr(value)?)
                    ));
                } else {
                    out.push_str(&format!(
                        "{}ahk.env.set(\"{}\", {});\n",
                        pad,
                        js_escape(&name),
                        self.emit_expr(value)?
                    ));
                }
            }
            Statement::Command { name, args } => {
                let (module, method) = commands::route(name);
                let rendered = args
                    .iter()
                    .map(|arg| match arg {
                        CommandArg::Literal(v) => format!("\"{}\"", js_escape(v)),
                        CommandArg::Expression(e) => {
                            self.emit_expr(e).unwrap_or_else(|_| "null".to_string())
                        }
                        CommandArg::OutputVar(v) => format!("\"{}\"", js_escape(v)),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!(
                    "{}ahk.{}.{}({});\n",
                    pad, module, method, rendered
                ));
            }
            Statement::ExprStatement(expr) => {
                out.push_str(&format!("{}{};\n", pad, self.emit_expr(expr)?));
            }
            Statement::If {
                condition,
                body,
                else_body,
            } => {
                out.push_str(&format!("{}if ({}) {{\n", pad, self.emit_expr(condition)?));
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                if let Some(else_body) = else_body {
                    out.push_str(&format!("{}}} else {{\n", pad));
                    for inner in else_body {
                        self.emit_statement(inner, indent + 1, out)?;
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::While { condition, body } => {
                out.push_str(&format!(
                    "{}while ({}) {{\n",
                    pad,
                    self.emit_expr(condition)?
                ));
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::For {
                key,
                value,
                iterable,
                body,
            } => {
                let value = value.clone().unwrap_or_else(|| "v".to_string());
                out.push_str(&format!(
                    "{}for (const [{}, {}] of Object.entries(({}) as Record<string, AhkValue>)) {{\n",
                    pad,
                    key,
                    value,
                    self.emit_expr(iterable)?
                ));
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::Loop { variant, body } => {
                let loop_footer = match variant {
                    LoopVariant::Count(expr) => {
                        let count = expr
                            .as_ref()
                            .map(|v| self.emit_expr(v))
                            .transpose()?
                            .unwrap_or_else(|| "0".to_string());
                        out.push_str(&format!(
                            "{}for (let A_Index: number = 1; A_Index <= Number({}); A_Index++) {{\n",
                            pad, count
                        ));
                        "}\n"
                    }
                    LoopVariant::Infinite => {
                        out.push_str(&format!("{}while (true) {{\n", pad));
                        "}\n"
                    }
                    LoopVariant::Parse { string, delimiters } => {
                        let s = self.emit_expr(string)?;
                        let d = delimiters
                            .as_ref()
                            .map(|v| self.emit_expr(v))
                            .transpose()?
                            .unwrap_or_else(|| "\",\"".to_string());
                        out.push_str(&format!(
                            "{}ahk.flow.loopParse({}, {}, () => {{\n",
                            pad, s, d
                        ));
                        "});\n"
                    }
                    LoopVariant::File { pattern, mode } => {
                        let p = self.emit_expr(pattern)?;
                        let m = mode
                            .as_ref()
                            .map(|v| self.emit_expr(v))
                            .transpose()?
                            .unwrap_or_else(|| "\"\"".to_string());
                        out.push_str(&format!(
                            "{}const _loopFiles: Array<any> = JSON.parse(String(ahk.disk.LoopFile({}, {}) || \"[]\"));\n",
                            pad, p, m
                        ));
                        out.push_str(&format!(
                            "{}for (let A_Index: number = 1; A_Index <= _loopFiles.length; A_Index++) {{\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const _entry: Record<string, AhkValue> = _loopFiles[A_Index - 1] ?? {{}};\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileName: AhkValue = _entry.name ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileExt: AhkValue = _entry.ext ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileFullPath: AhkValue = _entry.fullPath ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFilePath: AhkValue = _entry.fullPath ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileLongPath: AhkValue = _entry.longPath ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileShortPath: AhkValue = _entry.shortPath ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileShortName: AhkValue = _entry.shortName ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileDir: AhkValue = _entry.dir ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileTimeModified: AhkValue = _entry.timeModified ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileTimeCreated: AhkValue = _entry.timeCreated ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileTimeAccessed: AhkValue = _entry.timeAccessed ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileAttrib: AhkValue = _entry.attrib ?? \"\";\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileSize: AhkValue = _entry.size ?? 0;\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileSizeKB: AhkValue = _entry.sizeKB ?? 0;\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopFileSizeMB: AhkValue = _entry.sizeMB ?? 0;\n",
                            pad
                        ));
                        "}\n"
                    }
                    LoopVariant::Read { file } => {
                        let f = self.emit_expr(file)?;
                        out.push_str(&format!(
                            "{}const _loopReadLines: Array<any> = JSON.parse(String(ahk.disk.LoopReadFile({}) || \"[]\"));\n",
                            pad, f
                        ));
                        out.push_str(&format!(
                            "{}for (let A_Index: number = 1; A_Index <= _loopReadLines.length; A_Index++) {{\n",
                            pad
                        ));
                        out.push_str(&format!(
                            "{}  const A_LoopReadLine: AhkValue = _loopReadLines[A_Index - 1] ?? \"\";\n",
                            pad
                        ));
                        "}\n"
                    }
                    LoopVariant::Reg {
                        root_key,
                        key,
                        mode,
                    } => {
                        let k = key
                            .as_ref()
                            .map(|v| self.emit_expr(v))
                            .transpose()?
                            .unwrap_or_else(|| "\"\"".to_string());
                        let m = mode
                            .as_ref()
                            .map(|v| self.emit_expr(v))
                            .transpose()?
                            .unwrap_or_else(|| "\"\"".to_string());
                        out.push_str(&format!(
                            "{}ahk.flow.loopReg({}, {}, {}, () => {{\n",
                            pad,
                            self.emit_expr(root_key)?,
                            k,
                            m
                        ));
                        "});\n"
                    }
                };
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                out.push_str(&format!("{}{}", pad, loop_footer));
            }
            Statement::Break { .. } => out.push_str(&format!("{}break;\n", pad)),
            Statement::Continue { .. } => out.push_str(&format!("{}continue;\n", pad)),
            Statement::Until(expr) => {
                out.push_str(&format!("{}if ({}) break;\n", pad, self.emit_expr(expr)?))
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    out.push_str(&format!("{}return {};\n", pad, self.emit_expr(expr)?));
                } else {
                    out.push_str(&format!("{}return;\n", pad));
                }
            }
            Statement::Goto(label) => out.push_str(&format!(
                "{}ahk.flow.goto(\"{}\");\n",
                pad,
                js_escape(label)
            )),
            Statement::Gosub(label) => out.push_str(&format!(
                "{}ahk.flow.gosub(\"{}\");\n",
                pad,
                js_escape(label)
            )),
            Statement::Switch { value, cases } => {
                out.push_str(&format!("{}switch ({}) {{\n", pad, self.emit_expr(value)?));
                for case in cases {
                    if case.is_default {
                        out.push_str(&format!("{}  default:\n", pad));
                    } else {
                        for v in &case.values {
                            out.push_str(&format!("{}  case {}:\n", pad, self.emit_expr(v)?));
                        }
                    }
                    for inner in &case.body {
                        self.emit_statement(inner, indent + 2, out)?;
                    }
                    out.push_str(&format!("{}    break;\n", pad));
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::Try {
                body,
                catch,
                finally,
            } => {
                out.push_str(&format!("{}try {{\n", pad));
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                out.push_str(&format!("{}}}", pad));
                if let Some(catch) = catch {
                    let var_name = catch.var.clone().unwrap_or_else(|| "e".to_string());
                    out.push_str(&format!(" catch ({}: unknown) {{\n", var_name));
                    for inner in &catch.body {
                        self.emit_statement(inner, indent + 1, out)?;
                    }
                    out.push_str(&format!("{}}}", pad));
                }
                if let Some(finally) = finally {
                    out.push_str(" finally {\n");
                    for inner in finally {
                        self.emit_statement(inner, indent + 1, out)?;
                    }
                    out.push_str(&format!("{}}}\n", pad));
                } else {
                    out.push('\n');
                }
            }
            Statement::Throw(expr) => {
                out.push_str(&format!("{}throw {};\n", pad, self.emit_expr(expr)?))
            }
            Statement::VarDecl {
                scope,
                declarations,
            } => {
                let scope_name = match scope {
                    VarScope::Global => "declareGlobal",
                    VarScope::Local => "declareLocal",
                    VarScope::Static => "declareStatic",
                };
                for (name, value) in declarations {
                    match value {
                        Some(expr) => out.push_str(&format!(
                            "{}ahk.env.{}(\"{}\", {});\n",
                            pad,
                            scope_name,
                            js_escape(name),
                            self.emit_expr(expr)?
                        )),
                        None => out.push_str(&format!(
                            "{}ahk.env.{}(\"{}\");\n",
                            pad,
                            scope_name,
                            js_escape(name)
                        )),
                    }
                }
            }
            Statement::Block(inner) => {
                out.push_str(&format!("{}{{\n", pad));
                for statement in inner {
                    self.emit_statement(statement, indent + 1, out)?;
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::IfLegacy {
                variant,
                var,
                values,
                body,
                else_body,
            } => {
                let variant_name = format!("{:?}", variant);
                let args = values
                    .iter()
                    .map(|v| self.emit_expr(v))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                out.push_str(&format!(
                    "{}if (ahk.flow.ifLegacy(\"{}\", \"{}\", {})) {{\n",
                    pad,
                    variant_name,
                    js_escape(var),
                    args
                ));
                for inner in body {
                    self.emit_statement(inner, indent + 1, out)?;
                }
                if let Some(else_body) = else_body {
                    out.push_str(&format!("{}}} else {{\n", pad));
                    for inner in else_body {
                        self.emit_statement(inner, indent + 1, out)?;
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Statement::Empty => out.push_str(&format!("{};\n", pad)),
        }
        Ok(())
    }

    fn emit_expr(&self, expr: &Expr) -> Result<String, CodegenError> {
        let rendered = match expr {
            Expr::StringLiteral(value) => format!("\"{}\"", js_escape(value)),
            Expr::NumberLiteral(value) => value.to_string(),
            Expr::Variable(name) => format!("ahk.env.get(\"{}\")", js_escape(name)),
            Expr::Deref(name) => format!("ahk.env.get(\"{}\")", js_escape(name)),
            Expr::DoubleDeref(parts) => {
                let js = parts
                    .iter()
                    .map(|part| match part {
                        DerefPart::Literal(s) => format!("\"{}\"", js_escape(s)),
                        DerefPart::Variable(s) => format!("ahk.env.get(\"{}\")", js_escape(s)),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!("ahk.env.get({})", js)
            }
            Expr::UnaryOp { op, operand } => {
                let operand = self.emit_expr(operand)?;
                match op {
                    UnaryOp::Neg => format!("-({operand})"),
                    UnaryOp::Not | UnaryOp::LogicalNotKeyword => format!("!({operand})"),
                    UnaryOp::BitNot => format!("~({operand})"),
                    UnaryOp::AddressOf => format!("ahk.types.addressOf({operand})"),
                    UnaryOp::Deref => format!("ahk.types.deref({operand})"),
                    UnaryOp::PreInc => format!("++({operand})"),
                    UnaryOp::PreDec => format!("--({operand})"),
                    UnaryOp::PostInc => format!("({operand})++"),
                    UnaryOp::PostDec => format!("({operand})--"),
                }
            }
            Expr::BinaryOp { left, op, right } => {
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                let op = match op {
                    BinaryOp::Pow => "**",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::FloorDiv => "/",
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::ShiftLeft => "<<",
                    BinaryOp::ShiftRight => ">>",
                    BinaryOp::ShiftRightLogical => ">>>",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitXor => "^",
                    BinaryOp::BitOr => "|",
                    BinaryOp::RegexMatch => "~",
                    BinaryOp::Concat => "+",
                    BinaryOp::Eq | BinaryOp::StrictEq => "===",
                    BinaryOp::Neq | BinaryOp::StrictNeq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::Lte => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::Gte => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::Comma => ",",
                };
                if matches!(op, "~") {
                    format!("ahk.string.regexMatch({}, {})", left, right)
                } else {
                    format!("({left} {op} {right})")
                }
            }
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                format!(
                    "({} ? {} : {})",
                    self.emit_expr(condition)?,
                    self.emit_expr(then_branch)?,
                    self.emit_expr(else_branch)?
                )
            }
            Expr::FunctionCall { name, args } => {
                let args = args
                    .iter()
                    .map(|arg| self.emit_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                if let Expr::Variable(function_name) = name.as_ref() {
                    if let Some((module, method)) = commands::resolve(function_name) {
                        format!("ahk.{}.{}({})", module, method, args)
                    } else {
                        format!("{}({})", self.emit_expr(name)?, args)
                    }
                } else {
                    format!("{}({})", self.emit_expr(name)?, args)
                }
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let args = args
                    .iter()
                    .map(|arg| self.emit_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!("{}.{}({})", self.emit_expr(object)?, method, args)
            }
            Expr::PropertyAccess { object, property } => {
                format!("{}.{}", self.emit_expr(object)?, property)
            }
            Expr::IndexAccess { object, indices } => {
                let mut current = self.emit_expr(object)?;
                for i in indices {
                    current = format!("{}[{}]", current, self.emit_expr(i)?);
                }
                current
            }
            Expr::ObjectLiteral(items) => {
                let body = items
                    .iter()
                    .map(|(k, v)| Ok(format!("[{}]: {}", self.emit_expr(k)?, self.emit_expr(v)?)))
                    .collect::<Result<Vec<_>, CodegenError>>()?
                    .join(", ");
                format!("{{{body}}}")
            }
            Expr::ArrayLiteral(items) => {
                let body = items
                    .iter()
                    .map(|item| self.emit_expr(item))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!("[{body}]")
            }
            Expr::NewObject { class, args } => {
                let args = args
                    .iter()
                    .map(|arg| self.emit_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!("new {}({})", self.emit_expr(class)?, args)
            }
            Expr::Concatenation(parts) => {
                let body = parts
                    .iter()
                    .map(|part| self.emit_expr(part))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" + ");
                format!("({body})")
            }
            Expr::RegexMatch { left, right } => format!(
                "ahk.string.regexMatch({}, {})",
                self.emit_expr(left)?,
                self.emit_expr(right)?
            ),
            Expr::Comma(parts) => {
                let body = parts
                    .iter()
                    .map(|part| self.emit_expr(part))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!("({body})")
            }
            Expr::Base => "super".to_string(),
            Expr::This => "this".to_string(),
            Expr::Variadic(expr) => format!("...{}", self.emit_expr(expr)?),
            Expr::True => "true".to_string(),
            Expr::False => "false".to_string(),
            Expr::Null => "null".to_string(),
        };
        Ok(rendered)
    }

    fn emit_class(&self, class: &Class, out: &mut String) -> Result<(), CodegenError> {
        match &class.extends {
            Some(base) => out.push_str(&format!("class {} extends {} {{\n", class.name, base)),
            None => out.push_str(&format!("class {} {{\n", class.name)),
        }
        for member in &class.body {
            match member {
                ClassMember::Method(method) => {
                    out.push_str(&format!(
                        "  {}({}): AhkResult {{\n",
                        method.name,
                        self.emit_params(&method.params)
                    ));
                    out.push_str("    ahk.env.pushScope();\n");
                    for statement in &method.body {
                        self.emit_statement(statement, 2, out)?;
                    }
                    out.push_str("    ahk.env.popScope();\n");
                    out.push_str("  }\n");
                }
                ClassMember::Property {
                    name,
                    getter,
                    setter,
                } => {
                    out.push_str(&format!("  get {}(): AhkResult {{\n", name));
                    for statement in getter {
                        self.emit_statement(statement, 2, out)?;
                    }
                    out.push_str("  }\n");
                    out.push_str(&format!("  set {}(value: AhkValue) {{\n", name));
                    for statement in setter {
                        self.emit_statement(statement, 2, out)?;
                    }
                    out.push_str("  }\n");
                }
                ClassMember::InstanceVar { name, value } => {
                    let value = value
                        .as_ref()
                        .map(|e| self.emit_expr(e))
                        .transpose()?
                        .unwrap_or_else(|| "null".to_string());
                    out.push_str(&format!("  {}: AhkValue = {};\n", name, value));
                }
                ClassMember::StaticVar { name, value } => {
                    let value = value
                        .as_ref()
                        .map(|e| self.emit_expr(e))
                        .transpose()?
                        .unwrap_or_else(|| "null".to_string());
                    out.push_str(&format!("  static {}: AhkValue = {};\n", name, value));
                }
                ClassMember::NestedClass(nested) => {
                    self.emit_class(nested, out)?;
                }
            }
        }
        out.push_str("}\n");
        out.push_str(&format!(
            "ahk.types.registerClass(\"{}\", {});\n",
            js_escape(&class.name),
            class.name
        ));
        Ok(())
    }

    fn emit_params(&self, params: &[Param]) -> String {
        params
            .iter()
            .map(|param| {
                if param.is_variadic {
                    format!("...{}: AhkValue[]", param.name)
                } else {
                    format!("{}: AhkValue", param.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn emit_type_definitions(&self, out: &mut String) {
        out.push_str("type AhkPrimitive = string | number | boolean | null;\n");
        out.push_str("type AhkObject = { [key: string]: unknown };\n");
        out.push_str("type AhkArray = AhkValue[];\n");
        out.push_str("type AhkCallable = (...args: AhkValue[]) => AhkResult;\n");
        out.push_str("type AhkClass = new (...args: AhkValue[]) => AhkObject;\n");
        out.push_str("type AhkValue = AhkPrimitive | AhkObject | AhkArray | AhkCallable;\n");
        out.push_str("type AhkResult = AhkValue | void;\n");
        out.push_str("type AhkCallback = () => void;\n\n");

        out.push_str("interface AhkPlatform {\n");
        out.push_str("  readonly name: string;\n");
        out.push_str("  init(): void;\n");
        out.push_str("}\n\n");

        for module in modules::all() {
            let interface_name = module_interface_name(module.name);
            out.push_str(&format!("interface {} {{\n", interface_name));
            if module.name == "directives" {
                out.push_str("  [name: string]: (...args: AhkValue[]) => AhkResult;\n");
            }
            for &(method_name, _) in module.methods {
                if let Some(signature) = internal_signature(module.name, method_name) {
                    out.push_str("  ");
                    out.push_str(signature);
                    out.push_str("\n");
                } else {
                    out.push_str(&format!(
                        "  {}(...args: AhkValue[]): AhkResult;\n",
                        method_name
                    ));
                }
            }
            out.push_str("}\n\n");
        }

        out.push_str("interface AhkRuntime {\n");
        out.push_str("  readonly platform: AhkPlatform;\n");
        for module in modules::all() {
            out.push_str(&format!(
                "  readonly {}: {};\n",
                module.name,
                module_interface_name(module.name)
            ));
        }
        out.push_str("}\n\n");
        out.push_str("declare const ahk: AhkRuntime;\n\n");
    }

    fn emit_target_name(&self, target: &Expr) -> String {
        match target {
            Expr::Variable(name) => name.clone(),
            Expr::Deref(name) => name.clone(),
            _ => "_tmp".to_string(),
        }
    }

    fn modifier_name(&self, modifier: &Modifier) -> &'static str {
        match modifier {
            Modifier::Win => "Win",
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Shift => "Shift",
            Modifier::Left => "Left",
            Modifier::Right => "Right",
            Modifier::Wildcard => "Wildcard",
            Modifier::PassThrough => "PassThrough",
            Modifier::Hook => "Hook",
        }
    }
}

fn module_interface_name(module_name: &str) -> &'static str {
    match module_name {
        "env" => "AhkEnv",
        "ext" => "AhkExt",
        "disk" => "AhkDisk",
        "flow" => "AhkFlow",
        "gui" => "AhkGui",
        "maths" => "AhkMaths",
        "mnk" => "AhkMnk",
        "misc" => "AhkMisc",
        "types" => "AhkTypes",
        "process" => "AhkProcess",
        "registry" => "AhkRegistry",
        "screen" => "AhkScreen",
        "sound" => "AhkSound",
        "string" => "AhkString",
        "window" => "AhkWindow",
        "directives" => "AhkDirectives",
        _ => "AhkUnknown",
    }
}

fn internal_signature(module: &str, method: &str) -> Option<&'static str> {
    match (module, method) {
        ("env", "set") => Some("set(name: string, value: AhkValue): void;"),
        ("env", "get") => Some("get(name: string): AhkValue;"),
        ("env", "getBuiltIn") => Some("getBuiltIn(name: string): AhkValue;"),
        ("env", "pushScope") => Some("pushScope(): void;"),
        ("env", "popScope") => Some("popScope(): void;"),
        ("env", "declareGlobal") => Some("declareGlobal(name: string, value?: AhkValue): void;"),
        ("env", "declareLocal") => Some("declareLocal(name: string, value?: AhkValue): void;"),
        ("env", "declareStatic") => Some("declareStatic(name: string, value?: AhkValue): void;"),
        ("flow", "registerLabel") => Some("registerLabel(name: string, callback: AhkCallback): void;"),
        ("flow", "registerFunction") => Some("registerFunction(name: string, fn: AhkCallable): void;"),
        ("flow", "ifLegacy") => {
            Some("ifLegacy(variant: string, variable: string, ...values: AhkValue[]): boolean;")
        }
        ("flow", "loopParse") => {
            Some("loopParse(text: AhkValue, delimiters: AhkValue, callback: AhkCallback): void;")
        }
        ("flow", "loopFile") => {
            Some("loopFile(pattern: AhkValue, mode: AhkValue, callback: AhkCallback): void;")
        }
        ("flow", "loopRead") => Some("loopRead(file: AhkValue, callback: AhkCallback): void;"),
        ("flow", "loopReg") => {
            Some("loopReg(rootKey: AhkValue, key: AhkValue, mode: AhkValue, callback: AhkCallback): void;")
        }
        ("flow", "goto") => Some("goto(label: string): never;"),
        ("flow", "gosub") => Some("gosub(label: string): void;"),
        ("mnk", "registerHotkey") => {
            Some("registerHotkey(modifiers: string[], key: string, callback: AhkCallback): void;")
        }
        ("mnk", "registerHotstring") => {
            Some("registerHotstring(trigger: string, options: string, replacement: string | AhkCallback): void;")
        }
        ("types", "addressOf") => Some("addressOf(value: AhkValue): AhkValue;"),
        ("types", "deref") => Some("deref(value: AhkValue): AhkValue;"),
        ("types", "registerClass") => Some("registerClass(name: string, cls: AhkClass): void;"),
        ("string", "regexMatch") => Some("regexMatch(text: AhkValue, pattern: AhkValue): AhkValue;"),
        _ => None,
    }
}

fn js_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

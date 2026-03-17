use crate::ast::*;
use tree_sitter::Node;

pub struct Transformer<'a> {
    source: &'a str,
    _root: Node<'a>,
}

impl<'a> Transformer<'a> {
    pub fn new(source: &'a str, root: Node<'a>) -> Self {
        Self {
            source,
            _root: root,
        }
    }

    pub fn transform(&self) -> Script {
        let mut script = Script::default();
        for raw_line in self.source.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('#') {
                script.directives.push(self.parse_directive(line));
                continue;
            }
            if let Some(hotkey) = self.parse_hotkey(line) {
                script.hotkeys.push(hotkey);
                continue;
            }
            if let Some(hotstring) = self.parse_hotstring(line) {
                script.hotstrings.push(hotstring);
                continue;
            }
            if let Some(function) = self.parse_function_signature(line) {
                script.functions.push(function);
                continue;
            }
            if let Some(class) = self.parse_class_signature(line) {
                script.classes.push(class);
                continue;
            }
            if let Some(label) = self.parse_label(line) {
                script.labels.push(label);
                continue;
            }
            script.auto_exec.push(self.parse_statement(line));
        }
        script
    }

    fn parse_directive(&self, line: &str) -> Directive {
        let mut parts = line.split_whitespace();
        let name = parts
            .next()
            .unwrap_or("#Unknown")
            .trim_start_matches('#')
            .to_string();
        let args = parts.map(ToString::to_string).collect();
        Directive { name, args }
    }

    fn parse_hotkey(&self, line: &str) -> Option<Hotkey> {
        if !line.contains("::") || line.starts_with(':') {
            return None;
        }
        let head = line.split("::").next()?.trim();
        let mut modifiers = Vec::new();
        if head.contains('#') {
            modifiers.push(Modifier::Win);
        }
        if head.contains('!') {
            modifiers.push(Modifier::Alt);
        }
        if head.contains('^') {
            modifiers.push(Modifier::Ctrl);
        }
        if head.contains('+') {
            modifiers.push(Modifier::Shift);
        }
        if head.contains('*') {
            modifiers.push(Modifier::Wildcard);
        }
        if head.contains('~') {
            modifiers.push(Modifier::PassThrough);
        }
        if head.contains('$') {
            modifiers.push(Modifier::Hook);
        }
        let key = head
            .trim_matches(|c| "#!^+~$*<>".contains(c))
            .split('&')
            .next_back()
            .unwrap_or(head)
            .trim()
            .to_string();
        let custom_combo = if head.contains('&') {
            Some(head.to_string())
        } else {
            None
        };
        Some(Hotkey {
            modifiers,
            key,
            custom_combo,
            body: Vec::new(),
        })
    }

    fn parse_hotstring(&self, line: &str) -> Option<Hotstring> {
        if !line.starts_with(':') {
            return None;
        }
        let parts: Vec<&str> = line.split("::").collect();
        if parts.len() < 2 {
            return None;
        }
        let options = parts
            .first()
            .copied()
            .unwrap_or(":")
            .trim_start_matches(':')
            .to_string();
        let trigger = parts.get(1).copied().unwrap_or("").to_string();
        let replacement = parts
            .get(2)
            .map(|value| HotstringAction::Text((*value).to_string()))
            .unwrap_or_else(|| HotstringAction::Command(Vec::new()));
        Some(Hotstring {
            options,
            trigger,
            replacement,
        })
    }

    fn parse_function_signature(&self, line: &str) -> Option<Function> {
        if !line.ends_with('{') || !line.contains('(') || line.starts_with("if") {
            return None;
        }
        let signature = line.trim_end_matches('{').trim();
        let open = signature.find('(')?;
        let close = signature.rfind(')')?;
        let name = signature[..open].trim().to_string();
        let params = signature[open + 1..close]
            .split(',')
            .filter_map(|part| {
                let p = part.trim();
                if p.is_empty() {
                    return None;
                }
                let is_byref = p.starts_with("ByRef ");
                let is_variadic = p.ends_with('*');
                let name = p
                    .trim_start_matches("ByRef ")
                    .trim_end_matches('*')
                    .split(":=")
                    .next()
                    .unwrap_or(p)
                    .trim()
                    .to_string();
                let default = p
                    .split(":=")
                    .nth(1)
                    .map(|value| Expr::StringLiteral(value.trim().to_string()));
                Some(Param {
                    name,
                    is_byref,
                    default,
                    is_variadic,
                })
            })
            .collect();
        Some(Function {
            name,
            params,
            body: Vec::new(),
        })
    }

    fn parse_class_signature(&self, line: &str) -> Option<Class> {
        if !line.starts_with("class ") || !line.ends_with('{') {
            return None;
        }
        let signature = line.trim_end_matches('{').trim();
        let mut parts = signature.split_whitespace();
        let _class = parts.next();
        let name = parts.next()?.to_string();
        let extends = if parts.next() == Some("extends") {
            parts.next().map(ToString::to_string)
        } else {
            None
        };
        Some(Class {
            name,
            extends,
            body: Vec::new(),
        })
    }

    fn parse_label(&self, line: &str) -> Option<Label> {
        if line.ends_with(':') && !line.contains("::") {
            return Some(Label {
                name: line.trim_end_matches(':').to_string(),
                body: Vec::new(),
            });
        }
        None
    }

    fn parse_statement(&self, line: &str) -> Statement {
        if let Some(statement) = self.parse_var_decl(line) {
            return statement;
        }
        if let Some(statement) = self.parse_flow_statement(line) {
            return statement;
        }
        if let Some(statement) = self.parse_assignment(line) {
            return statement;
        }
        if let Some(statement) = self.parse_command(line) {
            return statement;
        }
        Statement::ExprStatement(self.parse_expr(line))
    }

    fn parse_var_decl(&self, line: &str) -> Option<Statement> {
        for (prefix, scope) in [
            ("global ", VarScope::Global),
            ("local ", VarScope::Local),
            ("static ", VarScope::Static),
        ] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let declarations = rest
                    .split(',')
                    .map(|chunk| {
                        let c = chunk.trim();
                        if let Some((name, value)) = c.split_once(":=") {
                            (name.trim().to_string(), Some(self.parse_expr(value.trim())))
                        } else if let Some((name, value)) = c.split_once('=') {
                            (
                                name.trim().to_string(),
                                Some(Expr::StringLiteral(value.trim().to_string())),
                            )
                        } else {
                            (c.to_string(), None)
                        }
                    })
                    .collect();
                return Some(Statement::VarDecl {
                    scope,
                    declarations,
                });
            }
        }
        None
    }

    fn parse_flow_statement(&self, line: &str) -> Option<Statement> {
        if line.starts_with("if ") {
            let condition = line.trim_start_matches("if").trim();
            return Some(Statement::If {
                condition: self.parse_expr(condition.trim_matches(|c| c == '(' || c == ')')),
                body: Vec::new(),
                else_body: None,
            });
        }
        if line.starts_with("while ") {
            return Some(Statement::While {
                condition: self.parse_expr(
                    line.trim_start_matches("while")
                        .trim()
                        .trim_matches(|c| c == '(' || c == ')'),
                ),
                body: Vec::new(),
            });
        }
        if line.starts_with("for ") {
            return Some(Statement::For {
                key: "k".to_string(),
                value: Some("v".to_string()),
                iterable: Expr::Variable("iterable".to_string()),
                body: Vec::new(),
            });
        }
        if line.starts_with("loop") {
            return Some(Statement::Loop {
                variant: LoopVariant::Infinite,
                body: Vec::new(),
            });
        }
        if line == "break" {
            return Some(Statement::Break { label: None });
        }
        if line == "continue" {
            return Some(Statement::Continue { label: None });
        }
        if let Some(rest) = line.strip_prefix("goto ") {
            return Some(Statement::Goto(rest.trim().to_string()));
        }
        if let Some(rest) = line.strip_prefix("gosub ") {
            return Some(Statement::Gosub(rest.trim().to_string()));
        }
        if line.starts_with("return") {
            let value = line.strip_prefix("return").unwrap_or("").trim();
            if value.is_empty() {
                return Some(Statement::Return(None));
            }
            return Some(Statement::Return(Some(self.parse_expr(value))));
        }
        if line.starts_with("throw ") {
            return Some(Statement::Throw(
                self.parse_expr(line.trim_start_matches("throw ").trim()),
            ));
        }
        None
    }

    fn parse_assignment(&self, line: &str) -> Option<Statement> {
        let ops = [
            (":=", AssignOp::Expr),
            ("+=", AssignOp::Add),
            ("-=", AssignOp::Sub),
            ("*=", AssignOp::Mul),
            ("/=", AssignOp::Div),
            ("//=", AssignOp::FloorDiv),
            (".=", AssignOp::Concat),
            ("|=", AssignOp::BitOr),
            ("&=", AssignOp::BitAnd),
            ("^=", AssignOp::BitXor),
            (">>=", AssignOp::ShiftRight),
            ("<<=", AssignOp::ShiftLeft),
            (">>>=", AssignOp::ShiftRightLogical),
            ("=", AssignOp::Legacy),
        ];
        for (marker, op) in ops {
            if let Some((lhs, rhs)) = line.split_once(marker) {
                let target = Expr::Variable(lhs.trim().to_string());
                let value = if matches!(op, AssignOp::Legacy) {
                    Expr::StringLiteral(rhs.trim().to_string())
                } else {
                    self.parse_expr(rhs.trim())
                };
                return Some(Statement::Assignment { target, op, value });
            }
        }
        None
    }

    fn parse_command(&self, line: &str) -> Option<Statement> {
        if line.contains('(') && line.ends_with(')') {
            return None;
        }
        let mut parts = line.splitn(2, ',');
        let head = parts.next()?.trim();
        if head.is_empty() {
            return None;
        }
        let name = head.split_whitespace().next()?.to_string();
        let args = parts
            .next()
            .map(|rest| {
                rest.split(',')
                    .map(|arg| CommandArg::Literal(arg.trim().to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Some(Statement::Command { name, args })
    }

    fn parse_expr(&self, text: &str) -> Expr {
        let value = text.trim();
        if value.is_empty() {
            return Expr::Null;
        }
        if value == "true" {
            return Expr::True;
        }
        if value == "false" {
            return Expr::False;
        }
        if value == "this" {
            return Expr::This;
        }
        if value == "base" {
            return Expr::Base;
        }
        if value.starts_with('"') && value.ends_with('"') {
            return Expr::StringLiteral(value.trim_matches('"').to_string());
        }
        if let Ok(num) = value.parse::<f64>() {
            return Expr::NumberLiteral(num);
        }
        if value.starts_with('%') && value.ends_with('%') && value.len() > 2 {
            return Expr::Deref(value.trim_matches('%').to_string());
        }
        if value.contains('.') {
            let items = value.split('.').map(|v| self.parse_expr(v)).collect();
            return Expr::Concatenation(items);
        }
        if value.contains('+') {
            let mut parts = value.splitn(2, '+');
            let left = self.parse_expr(parts.next().unwrap_or_default());
            let right = self.parse_expr(parts.next().unwrap_or_default());
            return Expr::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            };
        }
        if value.ends_with("()") {
            return Expr::FunctionCall {
                name: Box::new(Expr::Variable(value.trim_end_matches("()").to_string())),
                args: Vec::new(),
            };
        }
        Expr::Variable(value.to_string())
    }
}

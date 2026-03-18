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
        let mut cursor = self._root.walk();
        for node in self._root.named_children(&mut cursor) {
            match node.kind() {
                "comment" | "block_comment" | "doc_comment" => {}
                "directive" => script.directives.push(self.parse_directive_node(node)),
                "hotkey" => {
                    if let Some(hotkey) = self.parse_hotkey(self.node_text(node)) {
                        script.hotkeys.push(hotkey);
                    }
                }
                "hotstring_definition" => {
                    if let Some(hotstring) = self.parse_hotstring(self.node_text(node)) {
                        script.hotstrings.push(hotstring);
                    }
                }
                "function_definition" => {
                    if let Some(function) = self.parse_function_signature(self.node_text(node)) {
                        script.functions.push(function);
                    }
                }
                "class_definition" => {
                    if let Some(class) = self.parse_class_signature(self.node_text(node)) {
                        script.classes.push(class);
                    }
                }
                "label" => {
                    if let Some(label) = self.parse_label(self.node_text(node)) {
                        script.labels.push(label);
                    }
                }
                _ => script.auto_exec.push(self.parse_statement_node(node)),
            }
        }
        script
    }

    fn node_text(&self, node: Node<'a>) -> &'a str {
        node.utf8_text(self.source.as_bytes()).unwrap_or_default()
    }

    fn parse_directive_node(&self, node: Node<'a>) -> Directive {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(n).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let args = node
            .child_by_field_name("arguments")
            .map(|args| {
                self.node_text(args)
                    .split_whitespace()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Directive { name, args }
    }

    fn parse_statement_node(&self, node: Node<'a>) -> Statement {
        match node.kind() {
            "assignment_expression" => self.parse_assignment_node(node),
            "command" => self.parse_command_node(node),
            "if_statement" | "while_statement" | "for_statement" | "loop_statement"
            | "switch_statement" | "try_statement" | "return_statement" => {
                self.parse_statement(self.node_text(node))
            }
            _ => Statement::ExprStatement(self.parse_expr_node(node)),
        }
    }

    fn parse_assignment_node(&self, node: Node<'a>) -> Statement {
        let left_node = node.child_by_field_name("left");
        let right_node = node.child_by_field_name("right");

        let target = left_node
            .map(|left| self.parse_expr_node(left))
            .unwrap_or_else(|| Expr::Variable(String::new()));
        let value = right_node
            .map(|right| self.parse_expr_node(right))
            .unwrap_or(Expr::Null);

        let mut cursor = node.walk();
        let op = node
            .children(&mut cursor)
            .find_map(|child| match child.kind() {
                ":=" => Some(AssignOp::Expr),
                "+=" => Some(AssignOp::Add),
                "-=" => Some(AssignOp::Sub),
                "*=" => Some(AssignOp::Mul),
                "/=" => Some(AssignOp::Div),
                "//=" => Some(AssignOp::FloorDiv),
                ".=" => Some(AssignOp::Concat),
                "|=" => Some(AssignOp::BitOr),
                "&=" => Some(AssignOp::BitAnd),
                "^=" => Some(AssignOp::BitXor),
                ">>=" => Some(AssignOp::ShiftRight),
                "<<=" => Some(AssignOp::ShiftLeft),
                ">>>=" => Some(AssignOp::ShiftRightLogical),
                "=" => Some(AssignOp::Legacy),
                _ => None,
            })
            .unwrap_or(AssignOp::Expr);

        let value = if matches!(op, AssignOp::Legacy) {
            Expr::StringLiteral(
                self.node_text(right_node.unwrap_or(node))
                    .trim()
                    .to_string(),
            )
        } else {
            value
        };

        Statement::Assignment { target, op, value }
    }

    fn parse_command_node(&self, node: Node<'a>) -> Statement {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(n).to_string())
            .unwrap_or_default();

        let args_node = {
            let mut cursor = node.walk();
            let found = node
                .named_children(&mut cursor)
                .find(|child| child.kind() == "command_arguments");
            found
        };

        let args = args_node
            .map(|arguments| {
                let raw = self.node_text(arguments);
                self.split_top_level(raw, ',')
                    .into_iter()
                    .filter(|arg| !arg.is_empty())
                    .map(|arg| {
                        if arg.starts_with('%') && arg.ends_with('%') && arg.len() > 2 {
                            CommandArg::OutputVar(arg.trim_matches('%').to_string())
                        } else {
                            CommandArg::Literal(arg.trim().to_string())
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Statement::Command { name, args }
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

    fn split_top_level<'b>(&self, text: &'b str, delimiter: char) -> Vec<&'b str> {
        let mut parts = Vec::new();
        let mut start = 0usize;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut in_string = false;
        let mut chars = text.char_indices().peekable();

        while let Some((index, ch)) = chars.next() {
            match ch {
                '"' => in_string = !in_string,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string && paren_depth > 0 => paren_depth -= 1,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string && bracket_depth > 0 => bracket_depth -= 1,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string && brace_depth > 0 => brace_depth -= 1,
                _ => {}
            }

            if ch == delimiter
                && !in_string
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
            {
                parts.push(text[start..index].trim());
                start = index + ch.len_utf8();
            }
        }

        parts.push(text[start..].trim());
        parts
    }

    fn split_once_top_level<'b>(
        &self,
        text: &'b str,
        delimiter: char,
    ) -> Option<(&'b str, &'b str)> {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut in_string = false;

        for (index, ch) in text.char_indices() {
            match ch {
                '"' => in_string = !in_string,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string && paren_depth > 0 => paren_depth -= 1,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string && bracket_depth > 0 => bracket_depth -= 1,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string && brace_depth > 0 => brace_depth -= 1,
                _ => {}
            }

            if ch == delimiter
                && !in_string
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
            {
                let rhs_index = index + ch.len_utf8();
                return Some((text[..index].trim(), text[rhs_index..].trim()));
            }
        }

        None
    }

    fn parse_number_literal(&self, text: &str) -> Option<f64> {
        let value = text.trim();
        if value.is_empty() {
            return None;
        }

        let (sign, digits) = match value.as_bytes().first().copied() {
            Some(b'+') => (1.0, &value[1..]),
            Some(b'-') => (-1.0, &value[1..]),
            _ => (1.0, value),
        };

        if digits.starts_with("0x") || digits.starts_with("0X") {
            if digits.len() <= 2 {
                return None;
            }
            return i64::from_str_radix(&digits[2..], 16)
                .ok()
                .map(|number| sign * number as f64);
        }

        let is_scientific = digits.contains('e') || digits.contains('E');
        if is_scientific && !digits.contains('.') {
            return None;
        }

        if digits.chars().all(|ch| ch.is_ascii_digit()) || digits.contains('.') || is_scientific {
            return value.parse::<f64>().ok();
        }

        None
    }

    fn parse_function_call(&self, value: &str) -> Option<Expr> {
        if !value.ends_with(')') {
            return None;
        }

        let open = value.find('(')?;
        let name = value[..open].trim();
        if name.is_empty() || value[open + 1..value.len() - 1].contains('\n') {
            return None;
        }

        let args_text = &value[open + 1..value.len() - 1];
        let args = if args_text.trim().is_empty() {
            Vec::new()
        } else {
            self.split_top_level(args_text, ',')
                .into_iter()
                .map(|arg| self.parse_expr(arg))
                .collect()
        };

        Some(Expr::FunctionCall {
            name: Box::new(self.parse_expr(name)),
            args,
        })
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
        if let Some(num) = self.parse_number_literal(value) {
            return Expr::NumberLiteral(num);
        }
        if value.starts_with('%') && value.ends_with('%') && value.len() > 2 {
            return Expr::Deref(value.trim_matches('%').to_string());
        }
        if let Some((left, right)) = self.split_once_top_level(value, '+') {
            return Expr::BinaryOp {
                left: Box::new(self.parse_expr(left)),
                op: BinaryOp::Add,
                right: Box::new(self.parse_expr(right)),
            };
        }
        if let Some(function_call) = self.parse_function_call(value) {
            return function_call;
        }
        if let Some((object, property)) = self.split_once_top_level(value, '.') {
            return Expr::PropertyAccess {
                object: Box::new(self.parse_expr(object)),
                property: property.to_string(),
            };
        }
        Expr::Variable(value.to_string())
    }

    fn parse_expr_node(&self, node: Node<'a>) -> Expr {
        match node.kind() {
            "identifier" => Expr::Variable(self.node_text(node).to_string()),
            "number" => self
                .parse_number_literal(self.node_text(node))
                .map(Expr::NumberLiteral)
                .unwrap_or_else(|| Expr::Variable(self.node_text(node).to_string())),
            "string" => {
                let raw = self.node_text(node).trim();
                let normalized = if (raw.starts_with('"') && raw.ends_with('"'))
                    || (raw.starts_with('\'') && raw.ends_with('\''))
                {
                    raw[1..raw.len().saturating_sub(1)].to_string()
                } else {
                    raw.to_string()
                };
                Expr::StringLiteral(normalized)
            }
            "boolean" => match self.node_text(node).trim() {
                "true" => Expr::True,
                "false" => Expr::False,
                _ => Expr::Variable(self.node_text(node).to_string()),
            },
            "this_expression" => Expr::This,
            "base_expression" => Expr::Base,
            "variable_ref" => {
                let mut cursor = node.walk();
                let identifier = node
                    .named_children(&mut cursor)
                    .find(|child| child.kind() == "identifier")
                    .map(|child| self.node_text(child).to_string())
                    .unwrap_or_default();
                Expr::Deref(identifier)
            }
            "parenthesized_expression" => {
                let mut cursor = node.walk();
                let parsed = node
                    .named_children(&mut cursor)
                    .next()
                    .map(|child| self.parse_expr_node(child))
                    .unwrap_or(Expr::Null);
                parsed
            }
            "unary_expression" => {
                let mut cursor = node.walk();
                let mut op = UnaryOp::Neg;
                let mut operand = None;
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "-" => op = UnaryOp::Neg,
                        "!" | "not" => op = UnaryOp::Not,
                        "~" => op = UnaryOp::BitNot,
                        "&" => op = UnaryOp::AddressOf,
                        "*" => op = UnaryOp::Deref,
                        _ if child.is_named() => operand = Some(self.parse_expr_node(child)),
                        _ => {}
                    }
                }
                let operand = operand.unwrap_or(Expr::Null);
                if matches!(op, UnaryOp::Neg) {
                    if let Expr::NumberLiteral(value) = operand {
                        Expr::NumberLiteral(-value)
                    } else {
                        Expr::UnaryOp {
                            op,
                            operand: Box::new(operand),
                        }
                    }
                } else {
                    Expr::UnaryOp {
                        op,
                        operand: Box::new(operand),
                    }
                }
            }
            "binary_expression" => {
                let left = node
                    .child_by_field_name("left")
                    .map(|n| self.parse_expr_node(n))
                    .unwrap_or(Expr::Null);
                let right = node
                    .child_by_field_name("right")
                    .map(|n| self.parse_expr_node(n))
                    .unwrap_or(Expr::Null);

                let mut cursor = node.walk();
                let op = node
                    .children(&mut cursor)
                    .find_map(|child| match child.kind() {
                        "**" => Some(BinaryOp::Pow),
                        "*" => Some(BinaryOp::Mul),
                        "/" => Some(BinaryOp::Div),
                        "//" => Some(BinaryOp::FloorDiv),
                        "+" => Some(BinaryOp::Add),
                        "-" => Some(BinaryOp::Sub),
                        "<<" => Some(BinaryOp::ShiftLeft),
                        ">>" => Some(BinaryOp::ShiftRight),
                        ">>>" => Some(BinaryOp::ShiftRightLogical),
                        "&" => Some(BinaryOp::BitAnd),
                        "^" => Some(BinaryOp::BitXor),
                        "|" => Some(BinaryOp::BitOr),
                        "~=" => Some(BinaryOp::RegexMatch),
                        "." => Some(BinaryOp::Concat),
                        "=" | "==" => Some(BinaryOp::Eq),
                        "!=" | "<>" => Some(BinaryOp::Neq),
                        "<" => Some(BinaryOp::Lt),
                        "<=" => Some(BinaryOp::Lte),
                        ">" => Some(BinaryOp::Gt),
                        ">=" => Some(BinaryOp::Gte),
                        "&&" | "and" => Some(BinaryOp::And),
                        "||" | "or" => Some(BinaryOp::Or),
                        _ => None,
                    })
                    .unwrap_or(BinaryOp::Add);

                Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            "function_call" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| self.parse_expr_node(n))
                    .unwrap_or_else(|| Expr::Variable(String::new()));
                let args_node = {
                    let mut cursor = node.walk();
                    let found = node
                        .named_children(&mut cursor)
                        .find(|child| child.kind() == "argument_list");
                    found
                };
                let args = args_node
                    .map(|arguments| {
                        let mut cursor = arguments.walk();
                        arguments
                            .named_children(&mut cursor)
                            .map(|arg| self.parse_expr_node(arg))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Expr::FunctionCall {
                    name: Box::new(name),
                    args,
                }
            }
            "method_call" => {
                let object = node
                    .child_by_field_name("object")
                    .map(|n| self.parse_expr_node(n))
                    .unwrap_or(Expr::Null);
                let method = node
                    .child_by_field_name("property")
                    .map(|n| self.node_text(n).to_string())
                    .unwrap_or_default();
                let args_node = {
                    let mut cursor = node.walk();
                    let found = node
                        .named_children(&mut cursor)
                        .find(|child| child.kind() == "argument_list");
                    found
                };
                let args = args_node
                    .map(|arguments| {
                        let mut cursor = arguments.walk();
                        arguments
                            .named_children(&mut cursor)
                            .map(|arg| self.parse_expr_node(arg))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Expr::MethodCall {
                    object: Box::new(object),
                    method,
                    args,
                }
            }
            "member_expression" => {
                let object = node
                    .child_by_field_name("object")
                    .map(|n| self.parse_expr_node(n))
                    .unwrap_or(Expr::Null);
                let property = node
                    .child_by_field_name("property")
                    .map(|n| self.node_text(n).to_string())
                    .unwrap_or_default();
                Expr::PropertyAccess {
                    object: Box::new(object),
                    property,
                }
            }
            "concatenation_expression" => {
                let mut cursor = node.walk();
                let parts = node
                    .named_children(&mut cursor)
                    .map(|child| self.parse_expr_node(child))
                    .collect::<Vec<_>>();
                Expr::Concatenation(parts)
            }
            _ => self.parse_expr(self.node_text(node)),
        }
    }
}

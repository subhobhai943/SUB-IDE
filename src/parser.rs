use crate::lexer::{PythonIndentingLexer, Token, TokenKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyParameter {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyFunction {
    pub name: String,
    pub is_async: bool,
    pub parameters: Vec<PyParameter>,
    pub return_type: Option<String>,
    pub decorators: Vec<String>,
    pub docstring: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyClass {
    pub name: String,
    pub superclasses: Vec<String>,
    pub methods: Vec<PyFunction>,
    pub fields: Vec<String>,
    pub decorators: Vec<String>,
    pub docstring: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyImport {
    pub module: String,
    pub names: Vec<(String, Option<String>)>, // (imported_name, alias)
    pub is_from: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyFile {
    pub module_name: String,
    pub docstring: Option<String>,
    pub imports: Vec<PyImport>,
    pub classes: Vec<PyClass>,
    pub functions: Vec<PyFunction>,
}

/// Recursive-descent AST / PSI Parser for Python declarations.
/// Extracts high-level structural semantics (classes, functions, types, imports, docstrings).
pub struct PythonParser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl PythonParser {
    pub fn new(source: &str) -> Self {
        let mut lexer = PythonIndentingLexer::new(source);
        let tokens = lexer.tokenize_all();
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.cursor)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.cursor < self.tokens.len() {
            let tok = &self.tokens[self.cursor];
            self.cursor += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), TokenKind::Newline) {
            self.advance();
        }
    }

    /// Parse complete Python file into a PyFile PSI representation
    pub fn parse_file(&mut self, module_name: &str) -> PyFile {
        let mut docstring = None;
        let mut imports = Vec::new();
        let mut classes = Vec::new();
        let mut functions = Vec::new();

        self.skip_newlines();

        // Check module-level docstring
        if let TokenKind::StringLiteral(text) = self.peek() {
            docstring = Some(text.clone());
            self.advance();
            self.skip_newlines();
        }

        let mut pending_decorators = Vec::new();

        while self.peek() != &TokenKind::Eof {
            self.skip_newlines();

            match self.peek() {
                TokenKind::At => {
                    if let Some(dec) = self.parse_decorator() {
                        pending_decorators.push(dec);
                    }
                }
                TokenKind::Class => {
                    let decs = std::mem::take(&mut pending_decorators);
                    if let Some(py_class) = self.parse_class(decs) {
                        classes.push(py_class);
                    }
                }
                TokenKind::Def => {
                    let decs = std::mem::take(&mut pending_decorators);
                    if let Some(py_func) = self.parse_function(false, decs) {
                        functions.push(py_func);
                    }
                }
                TokenKind::Async => {
                    self.advance();
                    if self.peek() == &TokenKind::Def {
                        let decs = std::mem::take(&mut pending_decorators);
                        if let Some(py_func) = self.parse_function(true, decs) {
                            functions.push(py_func);
                        }
                    }
                }
                TokenKind::Import | TokenKind::From => {
                    pending_decorators.clear();
                    if let Some(imp) = self.parse_import() {
                        imports.push(imp);
                    }
                }
                _ => {
                    pending_decorators.clear();
                    self.advance();
                }
            }
        }

        PyFile {
            module_name: module_name.to_string(),
            docstring,
            imports,
            classes,
            functions,
        }
    }

    fn parse_decorator(&mut self) -> Option<String> {
        self.advance(); // consume @
        let mut dec_name = String::new();
        while let TokenKind::Identifier(name) = self.peek() {
            dec_name.push_str(name);
            self.advance();
            if self.peek() == &TokenKind::Dot {
                dec_name.push('.');
                self.advance();
            } else {
                break;
            }
        }

        // If decorator has arguments @deco(...), skip them
        if self.peek() == &TokenKind::LParen {
            self.advance();
            let mut depth = 1;
            while depth > 0 && self.peek() != &TokenKind::Eof {
                match self.advance().map(|t| &t.kind) {
                    Some(TokenKind::LParen) => depth += 1,
                    Some(TokenKind::RParen) => depth -= 1,
                    _ => {}
                }
            }
        }

        self.skip_newlines();
        if dec_name.is_empty() {
            None
        } else {
            Some(dec_name)
        }
    }

    fn parse_import(&mut self) -> Option<PyImport> {
        if self.peek() == &TokenKind::From {
            self.advance(); // consume 'from'
            let mut module = String::new();
            while let TokenKind::Identifier(name) = self.peek() {
                module.push_str(name);
                self.advance();
                if self.peek() == &TokenKind::Dot {
                    module.push('.');
                    self.advance();
                } else {
                    break;
                }
            }

            if self.peek() == &TokenKind::Import {
                self.advance(); // consume 'import'
            }

            let mut names = Vec::new();
            while let TokenKind::Identifier(name) = self.peek() {
                let imported = name.clone();
                self.advance();
                let mut alias = None;
                if self.peek() == &TokenKind::As {
                    self.advance();
                    if let TokenKind::Identifier(a) = self.peek() {
                        alias = Some(a.clone());
                        self.advance();
                    }
                }
                names.push((imported, alias));
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }

            Some(PyImport {
                module,
                names,
                is_from: true,
            })
        } else if self.peek() == &TokenKind::Import {
            self.advance(); // consume 'import'
            let mut module = String::new();
            while let TokenKind::Identifier(name) = self.peek() {
                module.push_str(name);
                self.advance();
                if self.peek() == &TokenKind::Dot {
                    module.push('.');
                    self.advance();
                } else {
                    break;
                }
            }

            let mut alias = None;
            if self.peek() == &TokenKind::As {
                self.advance();
                if let TokenKind::Identifier(a) = self.peek() {
                    alias = Some(a.clone());
                    self.advance();
                }
            }

            Some(PyImport {
                module: module.clone(),
                names: vec![(module, alias)],
                is_from: false,
            })
        } else {
            None
        }
    }

    fn parse_class(&mut self, decorators: Vec<String>) -> Option<PyClass> {
        let line_number = self.peek_token().map(|t| t.line).unwrap_or(0);
        self.advance(); // consume 'class'

        let class_name = match self.advance()?.kind {
            TokenKind::Identifier(ref name) => name.clone(),
            _ => return None,
        };

        let mut superclasses = Vec::new();
        if self.peek() == &TokenKind::LParen {
            self.advance();
            while let TokenKind::Identifier(base) = self.peek() {
                superclasses.push(base.clone());
                self.advance();
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek() == &TokenKind::RParen {
                self.advance();
            }
        }

        if self.peek() == &TokenKind::Colon {
            self.advance();
        }

        self.skip_newlines();

        let mut docstring = None;
        let mut methods = Vec::new();
        let mut fields = Vec::new();

        if self.peek() == &TokenKind::Indent {
            self.advance(); // consume Indent

            // Check class docstring
            if let TokenKind::StringLiteral(text) = self.peek() {
                docstring = Some(text.clone());
                self.advance();
                self.skip_newlines();
            }

            let mut pending_decs = Vec::new();

            while self.peek() != &TokenKind::Dedent && self.peek() != &TokenKind::Eof {
                self.skip_newlines();

                match self.peek() {
                    TokenKind::At => {
                        if let Some(dec) = self.parse_decorator() {
                            pending_decs.push(dec);
                        }
                    }
                    TokenKind::Def => {
                        let decs = std::mem::take(&mut pending_decs);
                        if let Some(method) = self.parse_function(false, decs) {
                            methods.push(method);
                        }
                    }
                    TokenKind::Async => {
                        self.advance();
                        if self.peek() == &TokenKind::Def {
                            let decs = std::mem::take(&mut pending_decs);
                            if let Some(method) = self.parse_function(true, decs) {
                                methods.push(method);
                            }
                        }
                    }
                    TokenKind::Identifier(field_name) => {
                        pending_decs.clear();
                        let f_name = field_name.clone();
                        self.advance();
                        if self.peek() == &TokenKind::Colon || self.peek() == &TokenKind::Assign {
                            fields.push(f_name);
                        }
                        // Skip rest of line
                        while self.peek() != &TokenKind::Newline
                            && self.peek() != &TokenKind::Dedent
                            && self.peek() != &TokenKind::Eof
                        {
                            self.advance();
                        }
                    }
                    _ => {
                        pending_decs.clear();
                        self.advance();
                    }
                }
            }

            if self.peek() == &TokenKind::Dedent {
                self.advance();
            }
        }

        Some(PyClass {
            name: class_name,
            superclasses,
            methods,
            fields,
            decorators,
            docstring,
            line_number,
        })
    }

    fn parse_function(&mut self, is_async: bool, decorators: Vec<String>) -> Option<PyFunction> {
        let line_number = self.peek_token().map(|t| t.line).unwrap_or(0);
        self.advance(); // consume 'def'

        let func_name = match self.advance()?.kind {
            TokenKind::Identifier(ref name) => name.clone(),
            _ => return None,
        };

        // Parse parameters
        let mut parameters = Vec::new();
        if self.peek() == &TokenKind::LParen {
            self.advance();
            while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                if let TokenKind::Identifier(param_name) = self.peek() {
                    let p_name = param_name.clone();
                    self.advance();

                    // Optional type annotation: param: int
                    let mut type_annotation = None;
                    if self.peek() == &TokenKind::Colon {
                        self.advance();
                        let mut type_str = String::new();
                        while self.peek() != &TokenKind::Comma
                            && self.peek() != &TokenKind::Assign
                            && self.peek() != &TokenKind::RParen
                            && self.peek() != &TokenKind::Eof
                        {
                            if let Some(t) = self.advance() {
                                type_str.push_str(&format!("{:?}", t.kind));
                            }
                        }
                        type_annotation = Some(type_str);
                    }

                    // Optional default value: = None
                    let mut default_value = None;
                    if self.peek() == &TokenKind::Assign {
                        self.advance();
                        if let Some(t) = self.advance() {
                            default_value = Some(format!("{:?}", t.kind));
                        }
                    }

                    parameters.push(PyParameter {
                        name: p_name,
                        type_annotation,
                        default_value,
                    });

                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }

            if self.peek() == &TokenKind::RParen {
                self.advance();
            }
        }

        // Return type annotation: -> ReturnType
        let mut return_type = None;
        if self.peek() == &TokenKind::Arrow {
            self.advance();
            let mut ret_str = String::new();
            while self.peek() != &TokenKind::Colon && self.peek() != &TokenKind::Eof {
                if let TokenKind::Identifier(id) = self.peek() {
                    ret_str.push_str(id);
                    self.advance();
                    if self.peek() == &TokenKind::Dot {
                        ret_str.push('.');
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }
            if !ret_str.is_empty() {
                return_type = Some(ret_str);
            }
        }

        if self.peek() == &TokenKind::Colon {
            self.advance();
        }

        self.skip_newlines();

        let mut docstring = None;

        // Function body
        if self.peek() == &TokenKind::Indent {
            self.advance(); // consume Indent

            // Check function docstring
            if let TokenKind::StringLiteral(text) = self.peek() {
                docstring = Some(text.clone());
                self.advance();
            }

            // Skip remaining function body tokens until Dedent
            let mut indent_depth = 1;
            while indent_depth > 0 && self.peek() != &TokenKind::Eof {
                match self.advance().map(|t| &t.kind) {
                    Some(TokenKind::Indent) => indent_depth += 1,
                    Some(TokenKind::Dedent) => indent_depth -= 1,
                    _ => {}
                }
            }
        }

        Some(PyFunction {
            name: func_name,
            is_async,
            parameters,
            return_type,
            decorators,
            docstring,
            line_number,
        })
    }
}

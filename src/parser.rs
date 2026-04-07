use crate::ast::*;
use crate::lexer::Token;
use crate::lexer_iter::Lexer;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    current: Option<(Token, std::ops::Range<usize>)>,
    source: &'source str,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next().and_then(
            |r: Result<(Token, std::ops::Range<usize>), crate::lexer::LexerError>| r.ok(),
        );
        Self {
            lexer,
            current,
            source,
        }
    }

    fn get_text(&self, span: &Span) -> &'source str {
        &self.source[span.start..span.end]
    }

    fn advance(&mut self) {
        self.current = self.lexer.next().and_then(
            |r: Result<(Token, std::ops::Range<usize>), crate::lexer::LexerError>| r.ok(),
        );
    }

    fn current_token(&self) -> Option<&Token> {
        self.current.as_ref().map(|(t, _)| t)
    }

    fn current_span(&self) -> Span {
        self.current
            .as_ref()
            .map(|(_, span)| Span {
                start: span.start,
                end: span.end,
            })
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn expect(&mut self, token: Token) -> Result<(Token, Span), String> {
        let tok = self.current_token().ok_or("Unexpected end of input")?;
        let tok_clone = *tok;
        if std::mem::discriminant(&tok_clone) == std::mem::discriminant(&token) {
            let span = self.current_span();
            self.advance();
            Ok((tok_clone, span))
        } else {
            Err(format!("Expected {:?} but got {:?}", token, tok_clone))
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();

        while self.current_token().is_some() {
            eprintln!("DEBUG: parse loop, token = {:?}", self.current_token());
            let token = self.current_token().unwrap();

            // Skip End token if present
            if token == &Token::End {
                self.advance();
                continue;
            }

            // Skip Semi token if present
            if token == &Token::Semi {
                self.advance();
                continue;
            }

            // Stop if no more valid source items (end of input or unexpected token)
            if !matches!(token, Token::Def | Token::Extern) {
                break;
            }

            match token {
                Token::Def => {
                    items.push(SourceItem::FuncDefinition(self.parse_function()?));
                }
                Token::Extern => {
                    items.push(SourceItem::FuncDeclaration(self.parse_declaration()?));
                }
                _ => {
                    return Err(format!(
                        "Expected function definition or declaration, got {:?}",
                        token
                    ));
                }
            }

            // Skip any trailing tokens
            while let Some(t) = self.current_token() {
                if t == &Token::End || t == &Token::Semi {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(Program { items })
    }

    fn parse_function(&mut self) -> Result<FuncDefinition, String> {
        let start = self.current_span();
        self.expect(Token::Def)?;
        let sig = self.parse_signature()?;

        eprintln!(
            "DEBUG: parse_function after signature, current = {:?}",
            self.current_token()
        );

        let mut body = Vec::new();

        // Parse body - can be either "begin ... end" or just statements ending with "end"
        if self.current_token() == Some(&Token::Begin)
            || self.current_token() == Some(&Token::LBrace)
        {
            let block_stmt = self.parse_block_like()?;
            if let Statement::Block(block) = block_stmt {
                body = block.body;
            }
            // After the block ends (e.g., 'end' after 'begin ... end'),
            // we need to consume the function's closing 'end'
            if self.current_token() == Some(&Token::End) {
                self.expect(Token::End)?;
            }
        } else {
            // Single statement without begin/end
            while let Some(tok) = self.current_token() {
                if tok == &Token::End {
                    self.expect(Token::End)?;
                    break;
                }
                if tok == &Token::Return {
                    let stmt = self.parse_statement()?;
                    body.push(stmt);
                } else {
                    // Skip unknown tokens
                    self.advance();
                }

                // Safety check - don't loop forever
                if body.len() > 10 {
                    break;
                }
            }

            // If we exited loop but didn't see End, try to consume it
            if self.current_token() == Some(&Token::End) {
                self.expect(Token::End)?;
            }
        }

        let span = start.merge(self.current_span());
        Ok(FuncDefinition {
            signature: sig,
            body,
            span,
        })
    }

    fn parse_declaration(&mut self) -> Result<FuncDeclaration, String> {
        let start = self.current_span();
        self.expect(Token::Extern)?;

        // Skip 'def' keyword if present
        if self.current_token() == Some(&Token::Def) {
            self.expect(Token::Def)?;
        }

        let sig = self.parse_signature()?;
        let span = start.merge(self.current_span());
        Ok(FuncDeclaration {
            signature: sig,
            span,
        })
    }

    fn parse_signature(&mut self) -> Result<FuncSignature, String> {
        let start = self.current_span();
        let (_name, name_span) = self.expect(Token::Identifier)?;
        let func_name = self.get_text(&name_span).to_string();

        self.expect(Token::LParen)?;
        let mut params = Vec::new();

        while self.current_token() != Some(&Token::RParen) {
            if !params.is_empty() {
                self.expect(Token::Comma)?;
            }

            let (_n, n_span) = self.expect(Token::Identifier)?;
            let param_name = self.get_text(&n_span).to_string();

            if self.current_token() == Some(&Token::Of) {
                self.expect(Token::Of)?;
            }

            let ty = self.parse_type()?;
            params.push(Arg {
                name: Identifier {
                    name: param_name,
                    span: start,
                },
                ty: Some(ty),
                span: start,
            });
        }

        self.expect(Token::RParen)?;

        let return_type = if self.current_token() == Some(&Token::Of) {
            self.expect(Token::Of)?;
            Some(self.parse_type()?)
        } else {
            None
        };

        let span = start.merge(self.current_span());
        Ok(FuncSignature {
            name: Identifier {
                name: func_name,
                span,
            },
            parameters: Some(params),
            return_type,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<TypeRef, String> {
        let start = self.current_span();
        let base_type = match self.current_token() {
            Some(Token::Int) => {
                self.advance();
                BuiltinType::Int
            }
            Some(Token::Uint) => {
                self.advance();
                BuiltinType::Uint
            }
            Some(Token::Long) => {
                self.advance();
                BuiltinType::Long
            }
            Some(Token::Ulong) => {
                self.advance();
                BuiltinType::Ulong
            }
            Some(Token::Byte) => {
                self.advance();
                BuiltinType::Byte
            }
            Some(Token::Bool) => {
                self.advance();
                BuiltinType::Bool
            }
            Some(Token::Char) => {
                self.advance();
                BuiltinType::Char
            }
            Some(Token::String) => {
                self.advance();
                BuiltinType::String
            }
            Some(Token::Array) => {
                self.advance();
                self.expect(Token::LBracket)?;
                let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                    let (_tok, span) = self.expect(Token::DecLiteral)?;
                    self.get_text(&span).parse().unwrap_or(0)
                } else {
                    0
                };
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                return Ok(TypeRef::Array {
                    element_type: Box::new(TypeRef::BuiltinType(BuiltinType::Int)),
                    size,
                    span,
                });
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                let name = self.get_text(&span).to_string();
                return Ok(TypeRef::Custom(Identifier { name, span }));
            }
            _ => return Err("Expected type".to_string()),
        };

        if self.current_token() == Some(&Token::Array) {
            self.advance();
            self.expect(Token::LBracket)?;
            let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                self.get_text(&span).parse().unwrap_or(0)
            } else {
                0
            };
            self.expect(Token::RBracket)?;
            let span = start.merge(self.current_span());
            return Ok(TypeRef::Array {
                element_type: Box::new(TypeRef::BuiltinType(base_type)),
                size,
                span,
            });
        }

        let _span = start.merge(self.current_span());
        Ok(TypeRef::BuiltinType(base_type))
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Some(Token::Return) => self.parse_return(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Break) => self.parse_break(),
            Some(Token::Begin) | Some(Token::LBrace) => self.parse_block_like(),
            Some(Token::Do) => self.parse_repeat(),
            Some(Token::Identifier) => self.parse_identifier_statement(),
            Some(Token::Assign) => Err("Unexpected assignment without variable".to_string()),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_identifier_statement(&mut self) -> Result<Statement, String> {
        // Currently at Identifier, need to check if next is Assign
        // Save current position by consuming identifier
        let (_name, span) = self.expect(Token::Identifier)?;
        let var_name = self.get_text(&span).to_string();

        // Check if next is assignment
        if self.current_token() == Some(&Token::Assign) {
            self.expect(Token::Assign)?;
            let expr = self.parse_expression(0)?;
            if self.current_token() == Some(&Token::Semi) {
                self.expect(Token::Semi)?;
            }
            let end_span = self.current_span();
            Ok(Statement::Expression(ExpressionStatement {
                expr: Expr::Binary(BinaryExpr {
                    left: Box::new(Expr::Identifier(Identifier {
                        name: var_name,
                        span,
                    })),
                    operator: BinaryOp::Assign,
                    right: Box::new(expr),
                    span: span.merge(end_span),
                }),
                span: span.merge(end_span),
            }))
        } else {
            // It's not an assignment, so treat as expression
            // We already consumed the identifier, so parse_expression will continue from there
            // But we need to handle this differently - let's create an identifier expression
            // and continue parsing as an expression
            let mut left = Expr::Identifier(Identifier {
                name: var_name,
                span,
            });

            // Continue parsing infix operators
            while let Some(token) = self.current_token() {
                if matches!(token, Token::Assign) {
                    break;
                }
                let token_copy = *token;
                let prec = Self::get_precedence(&token_copy);
                if prec == 0 {
                    break;
                }
                self.advance();
                left = self.parse_infix(left, token_copy, prec)?;
            }

            // Semi is optional
            if self.current_token() == Some(&Token::Semi) {
                self.expect(Token::Semi)?;
            }

            let end_span = self.current_span();
            Ok(Statement::Expression(ExpressionStatement {
                expr: left,
                span: span.merge(end_span),
            }))
        }
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Return)?;
        let expr = if self.current_token().is_some() && self.current_token() != Some(&Token::Semi) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        if self.current_token().is_some() {
            self.expect(Token::Semi)?;
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Return(ReturnStatement { expr, span }))
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::If)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::Then)?;
        let consequence = Box::new(self.parse_statement()?);
        let alternative = if self.current_token() == Some(&Token::Else) {
            self.expect(Token::Else)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        let span = start.merge(self.current_span());
        Ok(Statement::If(IfStatement {
            condition,
            consequence,
            alternative,
            span,
        }))
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::While)?;
        let condition = self.parse_expression(0)?;
        let _body = Box::new(self.parse_statement()?);
        let span = start.merge(self.current_span());
        Ok(Statement::Loop(LoopStatement {
            keyword: LoopKeyword::While,
            condition,
            body: vec![],
            span,
        }))
    }

    fn parse_repeat(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Do)?;
        let body = Box::new(self.parse_statement()?);
        self.expect(Token::While)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::Semi)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Repeat(RepeatStatement {
            body,
            keyword: LoopKeyword::Until,
            condition,
            span,
        }))
    }

    fn parse_break(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Break)?;
        self.expect(Token::Semi)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Break(BreakStatement { span }))
    }

    fn parse_block_like(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        eprintln!(
            "DEBUG: parse_block_like start, current = {:?}",
            self.current_token()
        );
        let end_token = match self.current_token() {
            Some(Token::Begin) => {
                self.expect(Token::Begin)?;
                Token::End
            }
            Some(Token::LBrace) => {
                self.expect(Token::LBrace)?;
                Token::RBrace
            }
            _ => return Err("Expected 'begin' or '{'".to_string()),
        };
        let mut body = Vec::new();
        loop {
            eprintln!("DEBUG: block loop, current = {:?}", self.current_token());
            // Check if we've reached the end token
            if let Some(tok) = self.current_token() {
                if std::mem::discriminant(tok) == std::mem::discriminant(&end_token) {
                    eprintln!("DEBUG: found end token, breaking");
                    break;
                }
            } else {
                // End of input before end token
                eprintln!("DEBUG: end of input before end token");
                break;
            }

            // Try to parse a statement
            match self.parse_statement() {
                Ok(stmt) => {
                    body.push(stmt);
                    // Skip trailing semicolons
                    while self.current_token() == Some(&Token::Semi) {
                        self.advance();
                    }
                }
                Err(e) => {
                    // If parsing fails, check if it's because we're at the end token
                    if let Some(tok) = self.current_token() {
                        if std::mem::discriminant(tok) == std::mem::discriminant(&end_token) {
                            break;
                        }
                    }
                    return Err(e);
                }
            }

            if body.len() > 100 {
                break;
            }
        }
        self.expect(end_token)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Block(BlockStatement { body, span }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        eprintln!(
            "DEBUG: parse_expression_statement start, current = {:?}",
            self.current_token()
        );
        let expr = self.parse_expression(0)?;
        // Semi is optional to allow block statements without trailing semicolon
        if self.current_token() == Some(&Token::Semi) {
            self.expect(Token::Semi)?;
        }
        let span = expr.span();
        Ok(Statement::Expression(ExpressionStatement { expr, span }))
    }

    fn parse_expression(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_prefix()?;

        while let Some(token) = self.current_token() {
            // Stop if we see an assignment operator - handle it at statement level
            if matches!(token, Token::Assign) {
                break;
            }
            let token_copy = *token;
            let prec = Self::get_precedence(&token_copy);
            if prec < min_prec {
                break;
            }
            self.advance();
            left = self.parse_infix(left, token_copy, prec)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        eprintln!(
            "DEBUG: parse_prefix start, current = {:?}",
            self.current_token()
        );
        match self.current_token() {
            Some(Token::DecLiteral) => {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                let value = self.get_text(&span).parse::<u64>().unwrap_or(0);
                Ok(Expr::Literal(Literal::Dec(value)))
            }
            Some(Token::HexLiteral) => {
                let (_tok, span) = self.expect(Token::HexLiteral)?;
                let text = self.get_text(&span);
                let value = u64::from_str_radix(&text[2..], 16).unwrap_or(0);
                Ok(Expr::Literal(Literal::Hex(value)))
            }
            Some(Token::BitsLiteral) => {
                let (_tok, span) = self.expect(Token::BitsLiteral)?;
                let text = self.get_text(&span);
                let value = u64::from_str_radix(&text[2..], 2).unwrap_or(0);
                Ok(Expr::Literal(Literal::Bits(value)))
            }
            Some(Token::StringLiteral) => {
                let (_tok, span) = self.expect(Token::StringLiteral)?;
                let s = self.get_text(&span);
                Ok(Expr::Literal(Literal::Str(s[1..s.len() - 1].to_string())))
            }
            Some(Token::CharLiteral) => {
                let (_tok, span) = self.expect(Token::CharLiteral)?;
                let s = self.get_text(&span);
                let ch = s.chars().nth(1).unwrap_or('\0');
                Ok(Expr::Literal(Literal::Char(ch)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                Ok(Expr::Identifier(Identifier {
                    name: self.get_text(&span).to_string(),
                    span,
                }))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect(Token::RParen)?;
                Ok(Expr::Parenthesized(Box::new(expr)))
            }
            Some(Token::Minus) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::Negate,
                    operand,
                    span,
                }))
            }
            Some(Token::Bang) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::Not,
                    operand,
                    span,
                }))
            }
            Some(Token::Tilde) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::BitNot,
                    operand,
                    span,
                }))
            }
            _ => Err("Expected expression".to_string()),
        }
    }

    fn parse_infix(&mut self, left: Expr, token: Token, prec: u8) -> Result<Expr, String> {
        eprintln!("DEBUG: parse_infix token = {:?}, prec = {}", token, prec);
        let start = left.span();

        match token {
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Lt
            | Token::Gt
            | Token::Le
            | Token::Ge
            | Token::Eq
            | Token::Ne
            | Token::And
            | Token::Or => {
                let right = self.parse_expression(prec + 1)?;
                let end = right.span();
                let op = match token {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Subtract,
                    Token::Star => BinaryOp::Multiply,
                    Token::Slash => BinaryOp::Divide,
                    Token::Percent => BinaryOp::Modulo,
                    Token::Lt => BinaryOp::Less,
                    Token::Gt => BinaryOp::Greater,
                    Token::Le => BinaryOp::LessOrEqual,
                    Token::Ge => BinaryOp::GreaterOrEqual,
                    Token::Eq => BinaryOp::Equal,
                    Token::Ne => BinaryOp::NotEqual,
                    Token::And => BinaryOp::And,
                    Token::Or => BinaryOp::Or,
                    _ => return Err("Unknown operator".to_string()),
                };
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    span: start.merge(end),
                }))
            }
            Token::LBracket => {
                let _index = self.parse_expression(0)?;
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                Ok(Expr::Slice(SliceExpr {
                    array: Box::new(left),
                    ranges: vec![],
                    span,
                }))
            }
            Token::LParen => {
                let mut args = Vec::new();
                while self.current_token() != Some(&Token::RParen) && self.current_token().is_some()
                {
                    if !args.is_empty() {
                        self.expect(Token::Comma)?;
                    }
                    args.push(self.parse_expression(0)?);
                }
                self.expect(Token::RParen)?;
                let span = start.merge(self.current_span());
                let func = Box::new(left);
                Ok(Expr::Call(CallExpr {
                    function: func,
                    arguments: args,
                    span,
                }))
            }
            Token::Assign => {
                let right = self.parse_expression(0)?;
                let span = start.merge(self.current_span());
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    operator: BinaryOp::Assign,
                    right: Box::new(right),
                    span,
                }))
            }
            _ => {
                // Not an infix operator - return the left side and restore position
                // This handles cases like function calls without parentheses, etc.
                Ok(left)
            }
        }
    }

    fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::Or => 10,
            Token::And => 20,
            Token::Eq | Token::Ne => 30,
            Token::Lt | Token::Gt | Token::Le | Token::Ge => 40,
            Token::Plus | Token::Minus => 50,
            Token::Star | Token::Slash | Token::Percent => 60,
            Token::Assign => 5,
            Token::LParen => 70,
            _ => 0,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Parenthesized(e) => e.span(),
            Expr::Call(e) => e.span,
            Expr::Slice(e) => e.span,
            Expr::Identifier(e) => e.span,
            Expr::Literal(_) => Span::new(0, 0),
        }
    }
}

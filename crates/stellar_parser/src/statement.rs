use stellar_ast::{
    token::{Keyword, Punctuator, RawToken},
    Statement,
};

use crate::{
    diagnostics::UnexpectedToken, expression::ExpressionParser, pattern::PatternParser,
    r#type::TypeParser, Parse, ParseState,
};

pub(crate) struct StatementParser;

pub(crate) struct StatementParserResult {
    pub(crate) statement: Statement,
    pub(crate) last_expression_in_block: bool,
}

impl Parse for StatementParser {
    type Output = Option<StatementParserResult>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let (statement, last_expression_in_block) = match state.next_token.raw {
            RawToken::Keyword(Keyword::Return) => (ReturnStatementParser.parse(state)?, false),
            RawToken::Keyword(Keyword::Defer) => (DeferStatementParser.parse(state)?, false),
            RawToken::Keyword(Keyword::Let) => (LetStatementParser.parse(state)?, false),
            RawToken::Keyword(Keyword::Continue) => (ContinueStatementParser.parse(state)?, false),
            RawToken::Keyword(Keyword::Break) => (BreakStatementParser.parse(state)?, false),
            _ => {
                let expression_statement_parser_result = ExpressionStatementParser.parse(state)?;

                (
                    expression_statement_parser_result.expression_statement,
                    expression_statement_parser_result.last_expression_in_block,
                )
            }
        };

        Some(StatementParserResult {
            statement,
            last_expression_in_block,
        })
    }
}

pub(crate) struct ExpressionStatementParser;

pub(crate) struct ExpressionStatementParserResult {
    pub(crate) expression_statement: Statement,
    pub(crate) last_expression_in_block: bool,
}

impl Parse for ExpressionStatementParser {
    type Output = Option<ExpressionStatementParserResult>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let expression = ExpressionParser::new().in_statements_block().parse(state)?;

        let (last_expression_in_block, has_semicolon) = if expression.with_block() {
            // 1. `ExpressionWithBlocks` are treated as individual statements
            //    (last_expression_in_block = false)
            // 2. Semicolons after them are also treated as individual statements
            //    (has_semicolon = false)
            (false, false)
        } else if state.next_token.raw == Punctuator::Semicolon {
            state.advance();

            (false, true)
        } else {
            (true, false)
        };

        Some(ExpressionStatementParserResult {
            expression_statement: Statement::Expression {
                expression,
                has_semicolon,
            },
            last_expression_in_block,
        })
    }
}

pub(crate) struct StatementsBlockParser;

impl Parse for StatementsBlockParser {
    type Output = Option<Vec<Statement>>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace)?;

        let mut block = vec![];

        loop {
            match state.next_token.raw {
                RawToken::Punctuator(Punctuator::CloseBrace) => break,
                RawToken::EndOfFile => {
                    state.diagnostics.add_diagnostic(UnexpectedToken::new(
                        state.current_token.location.end,
                        state.next_token,
                        Punctuator::CloseBrace,
                    ));

                    return None;
                }
                RawToken::Punctuator(Punctuator::Semicolon) => {
                    // Skip
                    state.advance();

                    continue;
                }
                _ => {}
            }

            let StatementParserResult {
                statement,
                last_expression_in_block,
            } = StatementParser.parse(state)?;
            block.push(statement);

            if last_expression_in_block {
                break;
            }
        }

        state.consume(Punctuator::CloseBrace)?;

        Some(block)
    }
}

struct DeferStatementParser;

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let call = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Defer { call })
    }
}

struct ReturnStatementParser;

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let expression = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Return { expression })
    }
}

struct LetStatementParser;

impl Parse for LetStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let pattern = PatternParser.parse(state)?;

        let ty = if state.next_token.raw == Punctuator::Colon {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        state.consume(Punctuator::Eq)?;

        let value = ExpressionParser::default().parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Let { pattern, value, ty })
    }
}

struct ContinueStatementParser;

impl Parse for ContinueStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let location = state.current_token.location;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Continue { location })
    }
}

struct BreakStatementParser;

impl Parse for BreakStatementParser {
    type Output = Option<Statement>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let location = state.current_token.location;

        state.consume(Punctuator::Semicolon)?;

        Some(Statement::Break { location })
    }
}

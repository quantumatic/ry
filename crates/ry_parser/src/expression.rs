use crate::{
    items::FunctionParameterParser,
    literal::LiteralParser,
    macros::parse_list,
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parse, TokenIterator,
};
use ry_ast::{
    precedence::Precedence, token::RawToken, BinaryOperator, IdentifierAst, MatchExpressionItem,
    PostfixOperator, PrefixOperator, RawBinaryOperator, RawPostfixOperator, RawPrefixOperator,
    StructExpressionItem, Token, UntypedExpression,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_source_file::span::Span;

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
    pub(crate) ignore_struct: bool,
}

struct WhileExpressionParser;

struct MatchExpressionParser;

struct MatchExpressionBlockParser;

struct MatchExpressionUnitParser;

struct PrimaryExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct GenericArgumentsExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct PropertyAccessExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct PrefixExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct PostfixExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct CastExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct IfExpressionParser;

struct ParenthesizedExpressionParser;

struct CallExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct BinaryExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
    pub(crate) ignore_struct: bool,
}

struct ListExpressionParser;

struct TupleExpressionParser;

struct StructExpressionParser {
    pub(crate) start: usize,
    pub(crate) left: UntypedExpression,
}

struct StructExpressionUnitParser;

struct FunctionExpressionParser;

struct StatementsBlockExpressionParser;

impl Parse for ExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        let mut left = PrimaryExpressionParser {
            ignore_struct: self.ignore_struct,
        }
        .parse_using(iterator)?;

        while self.precedence < iterator.next.raw.to_precedence() {
            left = match iterator.next.raw {
                Token!['('] => CallExpressionParser { start, left }.parse_using(iterator)?,
                Token![.] => {
                    PropertyAccessExpressionParser { start, left }.parse_using(iterator)?
                }
                Token!['['] => {
                    GenericArgumentsExpressionParser { start, left }.parse_using(iterator)?
                }
                Token![as] => CastExpressionParser { start, left }.parse_using(iterator)?,
                Token!['{'] => {
                    if self.ignore_struct {
                        return Some(left);
                    }

                    StructExpressionParser { start, left }.parse_using(iterator)?
                }
                _ => {
                    if iterator.next.raw.binary_operator() {
                        BinaryExpressionParser {
                            start,
                            left,
                            ignore_struct: self.ignore_struct,
                        }
                        .parse_using(iterator)?
                    } else if iterator.next.raw.postfix_operator() {
                        PostfixExpressionParser { start, left }.parse_using(iterator)?
                    } else {
                        break;
                    }
                }
            };
        }

        Some(left)
    }
}

impl Parse for WhileExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `while`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_using(iterator)?;

        let body = StatementsBlockParser.parse_using(iterator)?;

        Some(UntypedExpression::While {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            condition: Box::new(condition),
            body,
        })
    }
}

impl Parse for MatchExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `match`

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_using(iterator)?;

        let block = MatchExpressionBlockParser.parse_using(iterator)?;

        Some(UntypedExpression::Match {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            expression: Box::new(expression),
            block,
        })
    }
}

impl Parse for MatchExpressionBlockParser {
    type Output = Option<Vec<MatchExpressionItem>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.consume(Token!['{'], "match expression block")?;

        let units = parse_list!(iterator, "match expression block", Token!['}'], || {
            MatchExpressionUnitParser.parse_using(iterator)
        });

        iterator.next_token(); // `}`

        Some(units)
    }
}

impl Parse for MatchExpressionUnitParser {
    type Output = Option<MatchExpressionItem>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let left = PatternParser.parse_using(iterator)?;
        iterator.consume(Token![=>], "match expression unit")?;

        let right = ExpressionParser::default().parse_using(iterator)?;

        Some(MatchExpressionItem { left, right })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        match iterator.next.raw {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => Some(UntypedExpression::Literal(
                LiteralParser.parse_using(iterator)?,
            )),
            RawToken::Identifier => {
                let symbol = iterator.lexer.identifier();
                iterator.next_token();

                Some(UntypedExpression::Identifier(IdentifierAst {
                    span: iterator.current.span,
                    symbol,
                }))
            }
            Token!['('] => ParenthesizedExpressionParser.parse_using(iterator),
            Token!['['] => ListExpressionParser.parse_using(iterator),
            Token!['{'] => StatementsBlockExpressionParser.parse_using(iterator),
            Token![|] => FunctionExpressionParser.parse_using(iterator),
            Token![#] => TupleExpressionParser.parse_using(iterator),
            Token![if] => IfExpressionParser.parse_using(iterator),
            Token![match] => MatchExpressionParser.parse_using(iterator),
            Token![while] => WhileExpressionParser.parse_using(iterator),
            _ => {
                if iterator.next.raw.prefix_operator() {
                    return PrefixExpressionParser {
                        ignore_struct: self.ignore_struct,
                    }
                    .parse_using(iterator);
                }
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.next,
                        expected: expected!(
                            "integer literal",
                            "float literal",
                            "string literal",
                            "char literal",
                            "boolean literal",
                            Token![#],
                            Token![|],
                            Token!['('],
                            Token!['{'],
                            Token!['['],
                            "identifier",
                            Token![if],
                            Token![while],
                            Token![match]
                        ),
                        node: "expression".to_owned(),
                    }
                    .build(),
                );
                None
            }
        }
    }
}

impl Parse for GenericArgumentsExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let arguments = GenericArgumentsParser.parse_using(iterator)?;

        Some(UntypedExpression::GenericArguments {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token(); // `.`

        Some(UntypedExpression::Property {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            right: iterator.consume_identifier("property")?,
        })
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let operator_token = iterator.next;
        let operator: PrefixOperator = PrefixOperator {
            span: operator_token.span,
            raw: RawPrefixOperator::from(operator_token.raw),
        };
        iterator.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
            ignore_struct: self.ignore_struct,
        }
        .parse_using(iterator)?;

        Some(UntypedExpression::Prefix {
            span: Span::new(
                operator_token.span.start(),
                inner.span().end(),
                iterator.file_id,
            ),
            inner: Box::new(inner),
            operator,
        })
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let operator: PostfixOperator = PostfixOperator {
            span: iterator.current.span,
            raw: RawPostfixOperator::from(iterator.current.raw),
        };

        Some(UntypedExpression::Postfix {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            inner: Box::new(self.left),
            operator,
        })
    }
}

impl Parse for ParenthesizedExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: false,
        }
        .parse_using(iterator)?;

        iterator.consume(Token![')'], "parenthesized expression")?;

        Some(UntypedExpression::Parenthesized {
            span: Span::new(start, inner.span().end(), iterator.file_id),
            inner: Box::new(inner),
        })
    }
}

impl Parse for IfExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `if`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: true,
        }
        .parse_using(iterator)?;

        let block = StatementsBlockParser.parse_using(iterator)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while iterator.next.raw == Token![else] {
            iterator.next_token();

            if iterator.next.raw != Token![if] {
                r#else = Some(StatementsBlockParser.parse_using(iterator)?);
                break;
            }

            iterator.next_token();

            let condition = ExpressionParser {
                precedence: Precedence::Lowest,
                ignore_struct: true,
            }
            .parse_using(iterator)?;
            let block = StatementsBlockParser.parse_using(iterator)?;

            if_blocks.push((condition, block));
        }

        Some(UntypedExpression::If {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            if_blocks,
            r#else,
        })
    }
}

impl Parse for CastExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let right = TypeParser.parse_using(iterator)?;

        Some(UntypedExpression::As {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            right,
        })
    }
}

impl Parse for CallExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token(); // `(`

        let arguments = parse_list!(iterator, "call arguments list", Token![')'], || {
            ExpressionParser::default().parse_using(iterator)
        });

        iterator.next_token();

        Some(UntypedExpression::Call {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            arguments,
        })
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let operator_token = iterator.next;
        let operator: BinaryOperator = BinaryOperator {
            span: operator_token.span,
            raw: RawBinaryOperator::from(operator_token.raw),
        };
        let precedence = iterator.next.raw.to_precedence();

        iterator.next_token();

        let right = ExpressionParser {
            precedence,
            ignore_struct: self.ignore_struct,
        }
        .parse_using(iterator)?;

        Some(UntypedExpression::Binary {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        })
    }
}

impl Parse for ListExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let start = iterator.next.span.start();

        let elements = parse_list!(iterator, "list expression", Token![']'], || {
            ExpressionParser::default().parse_using(iterator)
        });

        iterator.next_token();

        Some(UntypedExpression::List {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            elements,
        })
    }
}

impl Parse for TupleExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `#`

        iterator.consume(Token!['('], "tuple expression")?;

        let elements = parse_list!(iterator, "tuple expression", Token![')'], || {
            ExpressionParser::default().parse_using(iterator)
        });

        iterator.next_token(); // `)`

        Some(UntypedExpression::Tuple {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            elements,
        })
    }
}

impl Parse for StructExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token(); // `{`

        let fields = parse_list!(iterator, "struct expression", Token!['}'], || {
            StructExpressionUnitParser.parse_using(iterator)
        });

        iterator.next_token(); // `}`

        Some(UntypedExpression::Struct {
            span: Span::new(self.start, iterator.current.span.end(), iterator.file_id),
            left: Box::new(self.left),
            fields,
        })
    }
}

impl Parse for StructExpressionUnitParser {
    type Output = Option<StructExpressionItem>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let name = iterator.consume_identifier("struct field")?;

        let value = if iterator.next.raw == Token![:] {
            iterator.next_token();
            Some(ExpressionParser::default().parse_using(iterator)?)
        } else {
            None
        };

        Some(StructExpressionItem { name, value })
    }
}

impl Parse for StatementsBlockExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        let block = StatementsBlockParser.parse_using(iterator)?;

        Some(UntypedExpression::StatementsBlock {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            block,
        })
    }
}

impl Parse for FunctionExpressionParser {
    type Output = Option<UntypedExpression>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next.span.start();
        iterator.next_token(); // `|`

        let parameters = parse_list!(
            iterator,
            "function expression parameters",
            Token![|],
            || { FunctionParameterParser.parse_using(iterator) }
        );

        iterator.next_token();

        let return_type = if iterator.next.raw == Token![:] {
            iterator.next_token();

            Some(TypeParser.parse_using(iterator)?)
        } else {
            None
        };

        let block = StatementsBlockParser.parse_using(iterator)?;

        Some(UntypedExpression::Function {
            span: Span::new(start, iterator.current.span.end(), iterator.file_id),
            parameters,
            return_type,
            block,
        })
    }
}

//! This crate provides a iter for Ry programming language
//!
//! It uses the lexer from the [`ry_lexer`] crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.
//!
//! [`ry_lexer`]: ../ry_lexer/index.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unnested_or_patterns
)]

mod expression;
mod items;
mod literal;
mod module;
mod path;
mod pattern;
mod statement;
mod r#type;

pub use module::{parse_module, parse_module_using};
use ry_ast::{
    token::{LexError, RawToken, Token},
    IdentifierAst, Token, Visibility,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic, CompilerDiagnostic};
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_workspace::{
    file::SourceFile,
    span::{Span, SpanIndex},
    workspace::FileID,
};

#[macro_use]
mod macros;

/// Represents a parse state.
#[derive(Debug)]
pub struct ParseState<'workspace, 'diagnostics, 'interner> {
    /// Source file, that is being parsed.
    source_file: &'workspace SourceFile<'workspace>,
    /// Id of the file in the global workspace.
    file_id: usize,
    /// Lexer that is used for parsing.
    lexer: Lexer<'workspace, 'interner>,
    /// Current token.
    current_token: Token,
    /// Next token.
    next_token: Token,
    /// Diagnostics that is emitted during parsing.
    diagnostics: &'diagnostics mut Vec<CompilerDiagnostic>,
}

/// Represents AST node that can be parsed.
pub trait Parse
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Parse AST node of type [`Self::Output`].
    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output;
}

/// Represents AST node that can optionally be parsed. Optionally
/// in this context means that if some condition is satisfied,
/// the AST node is parsed as usually (`Parse::parse(...)`),
/// but if not, it is skipped, token state is not advanced and the
/// default value is returned.
///
/// A great example of this is the where clause, which is found optional
/// in the syntax definition of every item in the Ry programming language.
/// To avoid copying the behaviour described below, this trait must
/// be implemented.
pub trait OptionalParser
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Optionally parse AST node of type [`Self::Output`].
    ///
    /// For more information, see [`OptionalParser`].
    fn optionally_parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output;
}

impl<'workspace, 'diagnostics, 'interner> ParseState<'workspace, 'diagnostics, 'interner> {
    /// Creates an initial parse state.
    #[must_use]
    pub fn new(
        file_id: FileID,
        source_file: &'workspace SourceFile<'workspace>,
        diagnostics: &'diagnostics mut Vec<CompilerDiagnostic>,
        interner: &'interner mut Interner,
    ) -> Self {
        let mut lexer = Lexer::new(file_id, source_file.source, interner);

        let current = lexer.next_no_comments();
        let next = current;

        let mut state = Self {
            source_file,
            file_id,
            lexer,
            current_token: current,
            next_token: next,
            diagnostics,
        };
        state.check_next_token();

        state
    }

    /// Adds diagnostic if the next token has lex error in itself.
    fn check_next_token(&mut self) {
        if let RawToken::Error(error) = self.next_token.raw {
            self.diagnostics.push(
                ParseDiagnostic::LexError(LexError {
                    span: self.next_token.span,
                    raw: error,
                })
                .build(),
            );
        }
    }

    /// Returns string slice corresponding to the given location.
    #[inline]
    #[must_use]
    fn resolve_span(&self, span: Span) -> &str {
        self.source_file.resolve_span(span)
    }

    /// Returns string slice corresponding to the current token's location.
    #[inline]
    #[must_use]
    fn resolve_current(&self) -> &str {
        self.resolve_span(self.current_token.span)
    }

    /// Advances the iter to the next token (skips comment tokens).
    fn advance(&mut self) {
        self.current_token = self.next_token;
        self.next_token = self.lexer.next_no_comments();
        self.check_next_token();
    }

    /// Checks if the next token is [`expected`].
    fn expect<N>(&mut self, expected: RawToken, node: N) -> Option<()>
    where
        N: Into<String>,
    {
        if self.next_token.raw == expected {
            Some(())
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next_token,
                    expected: expected!(expected),
                    node: node.into(),
                }
                .build(),
            );

            None
        }
    }

    /// Checks if the next token is [`expected`] and advances the parse state.
    fn consume<N>(&mut self, expected: RawToken, node: N) -> Option<()>
    where
        N: Into<String>,
    {
        self.expect(expected, node)?;
        self.advance();
        Some(())
    }

    /// Creates a new span with the state's file id.
    pub(crate) const fn new_span(&self, start: usize, end: usize) -> Span {
        Span {
            start,
            end,
            file_id: self.file_id,
        }
    }

    /// Creates a new span with the state's file id and
    /// ending with a current token span's end byte location.
    pub(crate) const fn span_to_current_from(&self, start: usize) -> Span {
        Span {
            start,
            end: self.current_token.span.end,
            file_id: self.file_id,
        }
    }

    /// Checks if the next token is identifiers, advances the parse state and if
    /// everything is ok, returns the identifier symbol.
    fn consume_identifier<N>(&mut self, node: N) -> Option<IdentifierAst>
    where
        N: Into<String>,
    {
        let spanned_symbol = if self.next_token.raw == RawToken::Identifier {
            IdentifierAst {
                span: self.next_token.span,
                symbol: self.lexer.scanned_identifier,
            }
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next_token,
                    expected: expected!("identifier"),
                    node: node.into(),
                }
                .build(),
            );
            return None;
        };

        self.advance();

        Some(spanned_symbol)
    }

    /// Consumes the identifiers and if the next token is not identifier than panics.
    pub(crate) fn consume_identifier_or_panic(&mut self) -> IdentifierAst {
        if self.next_token.raw == RawToken::Identifier {
            let identifier = IdentifierAst {
                span: self.next_token.span,
                symbol: self.lexer.scanned_identifier,
            };

            self.advance();

            identifier
        } else {
            unreachable!()
        }
    }

    /// Consumes the docstring for a module.
    pub(crate) fn consume_module_docstring(&mut self) -> Option<String> {
        if self.next_token.raw == RawToken::GlobalDocComment {
            let mut module_string = String::new();

            while self.next_token.raw == RawToken::GlobalDocComment {
                self.advance();

                module_string.push_str(self.source_file.source.index(self.current_token.span));
            }

            Some(module_string)
        } else {
            None
        }
    }

    /// Consumes the docstring for a local item.
    pub(crate) fn consume_local_docstring(&mut self) -> Option<String> {
        if self.next_token.raw == RawToken::LocalDocComment {
            let mut local_string = String::new();

            while self.next_token.raw == RawToken::LocalDocComment {
                self.advance();

                local_string.push_str(self.source_file.source.index(self.current_token.span));
            }

            Some(local_string)
        } else {
            None
        }
    }
}

pub(crate) struct VisibilityParser;

impl Parse for VisibilityParser {
    type Output = Visibility;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw == Token![pub] {
            state.advance();

            Visibility::public(state.current_token.span)
        } else {
            Visibility::private()
        }
    }
}

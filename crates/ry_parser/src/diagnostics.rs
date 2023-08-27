//! Defines diagnostics for parser.

#![allow(clippy::needless_pass_by_value)]

use std::fmt::Display;

use ry_ast::token::{LexError, Token};
use ry_diagnostics::{define_diagnostics, diagnostic::Diagnostic};
use ry_diagnostics::{BuildDiagnostic, LocationExt};
use ry_english_commons::enumeration;
use ry_filesystem::location::{ByteOffset, Location};
use ry_interner::PathID;

/// Represents list of expected tokens.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expected(pub Vec<String>);

/// Allows to construct [`Expected`] object shorter:
///
/// ```
/// use ry_parser::{expected, diagnostics::Expected};
///
/// assert_eq!(expected!("a", "b"), Expected(vec!["a".to_owned(), "b".to_owned()]));
/// ```
#[macro_export]
macro_rules! expected {
    ($($e:expr),*) => {{
        $crate::diagnostics::Expected(vec![$(format!("{}", $e)),*])
    }};
}

/// Context in which the unnecessary visibility qualifier error is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnnecessaryVisibilityQualifierContext {
    /// ```ry
    /// pub interface F {
    ///     pub fun t() {}
    ///     ^^^
    /// }
    /// ```
    InterfaceMethod {
        /// Location of a method name.
        name_location: Location,
    },

    /// ```ry
    /// pub import ...;
    /// ^^^
    /// ```
    Import,
}

define_diagnostics! {
    /// Diagnostic related to an error occured when tokenizing.
    diagnostic(error) LexErrorDiagnostic(self, error: LexError) {
        code { "E000" }
        message { format!("{}", self.error.raw) }
        labels {
            primary self.error.location => {""}
        }
        notes {}
    }

    /// Diagnostic related to an integer overflow error.
    diagnostic(error) IntegerOverflow(self, location: Location) {
        code { "E002" }
        message { "unexpected integer overflow" }
        labels {
            primary self.location => {"error appeared when parsing this integer"}
        }
        notes {
            "note: integer cannot exceed the maximum value of `u64` (u64.max() == 18_446_744_073_709_551_615)"
            "note: you can use exponent to do so, but be careful!"
        }
    }

    /// Diagnostic related to an float overflow error.
    diagnostic(error) FloatOverflow(self, location: Location) {
        code { "E003" }
        message { "unexpected float overflow" }
        labels {
            primary self.location => {"error appeared when parsing this float literal"}
        }
        notes {
            "note: float cannot exceed the maximum value of `f64` (f64.max() == 1.7976931348623157e+308)"
            "note: you can use exponent to do so, but be careful!"
        }
    }
}

/// Diagnostic related to an unexpected token error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedTokenDiagnostic {
    /// End byte offset of the token before unexpected one.
    pub offset: Option<ByteOffset>,

    /// The token that was not expected.
    pub got: Token,

    /// Tokens that were expected.
    pub expected: Expected,

    /// AST Node at which the error occurred while parsing.
    pub node: String,
}

impl UnexpectedTokenDiagnostic {
    /// Creates a new instance of [`UnexpectedTokenDiagnostic`].
    #[inline(always)]
    #[must_use]
    pub fn new(
        offset: Option<ByteOffset>,
        got: Token,
        expected: Expected,
        node: impl Into<String>,
    ) -> Self {
        Self {
            offset,
            got,
            expected,
            node: node.into(),
        }
    }
}

impl BuildDiagnostic for UnexpectedTokenDiagnostic {
    #[inline(always)]
    fn build(self) -> Diagnostic<PathID> {
        Diagnostic::error()
            .with_message(format!(
                "expected {}, found {}",
                self.expected, self.got.raw
            ))
            .with_code("E001")
            .with_labels(if let Some(offset) = self.offset {
                vec![
                    offset
                        .next_byte_location_at(self.got.location.file_path_id)
                        .to_secondary_label()
                        .with_message(format!("expected {}", self.expected)),
                    self.got
                        .location
                        .to_primary_label()
                        .with_message(format!("found {}", self.got.raw)),
                ]
            } else {
                vec![self
                    .got
                    .location
                    .to_primary_label()
                    .with_message(format!("expected {} for {}", self.expected, self.node))]
            })
    }
}

/// Diagnostic related to an unnecessary visibility qualifier error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnnecessaryVisibilityQualifierDiagnostic {
    /// Location of `pub`.
    pub location: Location,

    /// Context in which the error is found.
    pub context: UnnecessaryVisibilityQualifierContext,
}

impl BuildDiagnostic for UnnecessaryVisibilityQualifierDiagnostic {
    #[inline(always)]
    fn build(self) -> Diagnostic<PathID> {
        let mut labels = vec![self
            .location
            .to_primary_label()
            .with_message("consider removing this `pub`")];

        if let UnnecessaryVisibilityQualifierContext::InterfaceMethod { name_location } =
            self.context
        {
            labels.push(
                name_location
                    .to_secondary_label()
                    .with_message("happened when analyzing the interface method"),
            );
        }

        Diagnostic::error()
            .with_message("unnecessary visibility qualifier".to_owned())
            .with_code("E004")
            .with_labels(labels)
            .with_notes(match self.context {
                UnnecessaryVisibilityQualifierContext::InterfaceMethod { .. } => {
                    vec![
                        "note: using `pub` for interface method will not make the method public"
                            .to_owned(),
                        "note: all interface methods are public by default".to_owned(),
                    ]
                }
                UnnecessaryVisibilityQualifierContext::Import => {
                    vec!["note: using `pub` will not make the import public.".to_owned()]
                }
            })
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&enumeration::one_of(self.0.iter(), false))
    }
}

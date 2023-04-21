use crate::{error::ParseResult, path::PathParser, Parser, ParserState};
use ry_ast::{
    declaration::{ImportItem, Item},
    Token, Visibility,
};

#[derive(Default)]
pub(crate) struct ImportParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImportParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let path = PathParser.parse_with(state)?;
        state.consume(Token![;], "import")?;

        Ok(ImportItem {
            visibility: self.visibility,
            path,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(ImportParser, single_import, "import test;");
    parser_test!(ImportParser, imports, "import test; import test2.test;");
}
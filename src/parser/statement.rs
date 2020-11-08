//! Parsing for JS statements.

use crate::ast::*;
use crate::parser::util::*;
use crate::parser::*;
use nom::{branch::alt, bytes::complete::*, combinator::*};
use nom_locate::position;

pub fn parse_stmt(s: Span) -> ParseResult<Node> {
    alt((parse_block, parse_var_stmt, parse_empty_stmt))(s)
}

pub fn parse_block(s: Span) -> ParseResult<Node> {
    let (s, start) = position(s)?;
    let (s, body) = delimited(ws0(tag("{")), parse_stmt_list, tag("}"))(s)?;
    let (s, end) = ws0(position)(s)?;

    Ok((s, NodeKind::BlockStatement { body }.with_pos(start, end)))
}

pub fn parse_stmt_list(s: Span) -> ParseResult<Vec<Node>> {
    many0(parse_stmt)(s)
}

pub fn parse_var_stmt(s: Span) -> ParseResult<Node> {
    let parse_initializer = preceded(ws0(tag("=")), parse_expr_no_seq);
    let parse_var_declaration = map(
        tuple((
            position,
            pair(parse_identifier, opt(parse_initializer)),
            position,
        )),
        |(start, (id, init), end)| {
            NodeKind::VariableDeclarator {
                id: Box::new(id),
                init: Box::new(init),
            }
            .with_pos(start, end)
        },
    );
    let mut parse_declaration_list = separated_list1(ws0(tag(",")), parse_var_declaration);

    let (s, start) = position(s)?;
    let (s, _) = ws0(keyword_var)(s)?;

    let (s, declarations) = parse_declaration_list(s)?;

    let (s, _) = opt(semi)(s)?;
    let (s, end) = ws0(position)(s)?;
    Ok((
        s,
        NodeKind::VariableDeclaration {
            declarations,
            kind: VariableDeclarationKind::Var,
        }
        .with_pos(start, end),
    ))
}

pub fn parse_empty_stmt(s: Span) -> ParseResult<Node> {
    let (s, start) = position(s)?;
    let (s, _) = tag(";")(s)?;
    let (s, end) = position(s)?;
    Ok((s, NodeKind::EmptyStatement.with_pos(start, end)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_var_stmt() {
        assert_json_snapshot!(parse_stmt("var x = 1;".into()).unwrap().1);
        assert_json_snapshot!(parse_stmt("var x, y = 1;".into()).unwrap().1);
        assert_json_snapshot!(parse_stmt("var x = 1, y;".into()).unwrap().1);
        assert_json_snapshot!(parse_stmt("var x".into()).unwrap().1); // auto insert semi
    }

    #[test]
    fn test_empty_stmt() {
        assert_eq!(
            parse_empty_stmt(";".into()).unwrap().1.kind,
            NodeKind::EmptyStatement
        );
    }
}
use pest::{Parser, iterators::Pair, pratt_parser::PrattParser};
use pest_derive::Parser;
use std::sync::LazyLock;

use crate::ast::*;

pub fn parse(code: &str) -> Result<Program, ::pest::error::Error<Rule>> {
    match dbg!(ExpLang::parse(Rule::file, &code)) {
        Ok(mut pairs) => Ok(parse_program(pairs.next().unwrap())),
        Err(err) => Err(err),
    }
}

fn parse_program(pair: Pair<Rule>) -> Program {
    let mut statements = Vec::new();
    for statement in pair.into_inner() {
        statements.push(parse_statement(statement));
    }
    Program { statements }
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::func_def => {
            let mut pairs = pair.into_inner();
            let ident = parse_ident(pairs.next().unwrap());
            let args = pairs
                .next()
                .unwrap()
                .into_inner()
                .map(|pair| parse_ident(pair))
                .collect();
            let expr = parse_expr(pairs.next().unwrap());
            Statement::FuncDef { ident, args, expr }
        }
        Rule::Let => {
            let mut pairs = pair.into_inner();
            let ident = parse_ident(pairs.next().unwrap());
            let value = parse_expr(pairs.next().unwrap());
            Statement::Let { ident, value }
        }
        Rule::For => {
            let mut pairs = pair.into_inner();
            let ident = parse_ident(pairs.next().unwrap());
            let iter = parse_expr(pairs.next().unwrap());
            let expr = parse_expr(pairs.next().unwrap());
            Statement::For { ident, iter, expr }
        }
        Rule::expr => Statement::Expr(parse_expr(pair)),
        _ => unreachable!(),
    }
}

fn parse_ident(pair: Pair<Rule>) -> Ident {
    Ident(pair.as_str().to_string())
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::none => Expr::Literal(Literal::None),
            Rule::number => Expr::Literal(Literal::Number(primary.as_str().parse().unwrap())),
            Rule::bool => Expr::Literal(Literal::Bool(
                primary.into_inner().next().unwrap().as_rule() == Rule::True,
            )),
            Rule::string => Expr::Literal(Literal::String(
                primary.into_inner().next().unwrap().as_str().to_string(),
            )),
            Rule::array => Expr::Literal(Literal::Array(
                primary.into_inner().map(|pair| parse_expr(pair)).collect(),
            )),
            Rule::ident => Expr::Literal(Literal::Ident(parse_ident(primary))),
            Rule::If => {
                let mut pairs = primary.into_inner();
                let condition = Box::new(parse_expr(pairs.next().unwrap()));
                let expr = Box::new(parse_expr(pairs.next().unwrap()));
                let or = if let Some(or) = pairs.next() {
                    Some(Box::new(parse_expr(or)))
                } else {
                    None
                };
                Expr::Literal(Literal::If {
                    condition,
                    expr,
                    or,
                })
            }
            Rule::block => {
                let mut pairs = primary.into_inner();
                let statements = pairs
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(|pair| parse_statement(pair))
                    .collect();
                let return_value = if let Some(retrun_value) = pairs.next() {
                    Box::new(parse_expr(retrun_value))
                } else {
                    Box::new(Expr::Literal(Literal::None))
                };
                Expr::Literal(Literal::Block {
                    statements,
                    return_value,
                })
            }
            Rule::expr => Expr::Expr(Box::new(parse_expr(dbg!(primary)))),
            _ => unreachable!(),
        })
        .map_infix(|lhs, operator, rhs| {
            let operator = match operator.as_rule() {
                Rule::add => BinaryOperator::Add,
                Rule::subtract => BinaryOperator::Subtract,
                Rule::multiply => BinaryOperator::Multiply,
                Rule::divide => BinaryOperator::Divide,
                Rule::modulo => BinaryOperator::Modulo,
                Rule::range => BinaryOperator::Range,
                Rule::equal => BinaryOperator::Equal,
                Rule::assign => BinaryOperator::Assign,
                _ => unreachable!(),
            };
            Expr::Binary {
                lhs: Box::new(lhs),
                operator,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|operator, operand| {
            let operator = match operator.as_rule() {
                Rule::minus => UnaryOperator::Minus,
                _ => unreachable!(),
            };
            Expr::Unary {
                operand: Box::new(operand),
                operator,
            }
        })
        .map_postfix(|operand, operator| {
            let operator = match operator.as_rule() {
                Rule::call => UnaryOperator::Call(
                    operator.into_inner().map(|pair| parse_expr(pair)).collect(),
                ),
                Rule::index => UnaryOperator::Index(Box::new(parse_expr(
                    operator.into_inner().next().unwrap(),
                ))),
                _ => unreachable!(),
            };
            Expr::Unary {
                operand: Box::new(operand),
                operator,
            }
        })
        .parse(pair.into_inner())
}

#[derive(Parser)]
#[grammar = "exp.pest"]
struct ExpLang;

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use pest::pratt_parser::{Assoc, Op};
    PrattParser::new()
        .op(Op::infix(Rule::assign, Assoc::Right))
        .op(Op::infix(Rule::equal, Assoc::Left))
        .op(Op::infix(Rule::range, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::subtract, Assoc::Left))
        .op(Op::infix(Rule::multiply, Assoc::Left)
            | Op::infix(Rule::divide, Assoc::Left)
            | Op::infix(Rule::modulo, Assoc::Left))
        .op(Op::prefix(Rule::minus))
        .op(Op::postfix(Rule::call) | Op::postfix(Rule::index))
});

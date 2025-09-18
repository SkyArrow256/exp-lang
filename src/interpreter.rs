use rustc_hash::FxHashMap;

use crate::ast::*;
use std::mem;

pub fn eval(program: Program) -> Primitive {
    let mut global_scope = Scope::new();
    program.statements.iter().for_each(|statement| {
        eval_statement(statement, &mut global_scope);
    });
    let main = Expr::Unary {
        operand: Box::new(Expr::Literal(Literal::Ident(Ident(String::from("main"))))),
        operator: UnaryOperator::Call(Vec::new()),
    };
    eval_expr(&main, &mut global_scope)
}

fn eval_statement(statement: &Statement, scope: &mut Scope) {
    match statement {
        Statement::Expr(expr) => {
            eval_expr(expr, scope);
        }
        Statement::Let { ident, value } => {
            let value = eval_expr(value, scope);
            scope.add(ident.clone(), value);
        }
        Statement::For { ident, iter, expr } => {
            if let Primitive::Array(iter) = eval_expr(iter, scope) {
                for element in iter.into_iter() {
                    scope.create();
                    scope.add(ident.clone(), element);
                    eval_expr(expr, scope);
                    scope.delete();
                }
            } else {
                panic!("TypeError");
            }
        }
        Statement::FuncDef { ident, args, expr } => {
            scope.add(
                ident.clone(),
                Primitive::Function {
                    args_name: args.to_owned(),
                    code: expr.clone(),
                },
            );
        }
    }
}

fn eval_expr(expr: &Expr, scope: &mut Scope) -> Primitive {
    match expr {
        Expr::Literal(literal) => match literal {
            Literal::Number(num) => Primitive::Number(*num),
            Literal::Bool(bool) => Primitive::Bool(*bool),
            Literal::String(str) => Primitive::String(str.clone()),
            Literal::Array(array) => {
                Primitive::Array(array.iter().map(|expr| eval_expr(expr, scope)).collect())
            }
            Literal::Ident(ident) => scope.get(ident).unwrap().clone(),
            Literal::Block {
                statements,
                return_value,
            } => {
                scope.create();
                statements
                    .iter()
                    .for_each(|statement| eval_statement(statement, scope));
                let result = eval_expr(return_value, scope);
                scope.delete();
                result
            }
            Literal::If {
                condition,
                expr,
                or,
            } => {
                if let Primitive::Bool(bool) = eval_expr(condition, scope) {
                    if bool {
                        eval_expr(expr, scope)
                    } else {
                        if let Some(or_expr) = or {
                            eval_expr(or_expr, scope)
                        } else {
                            Primitive::None
                        }
                    }
                } else {
                    panic!("TypeError: bool型が予期されましたがそれ以外でした");
                }
            }
            Literal::None => Primitive::None,
        },
        Expr::Unary {
            operand: operand_exp,
            operator,
        } => {
            let operand = eval_expr(operand_exp, scope);
            match operator {
                UnaryOperator::Minus => {
                    if let Primitive::Number(operand) = operand {
                        Primitive::Number(-operand)
                    } else {
                        panic!("TypeError");
                    }
                }
                UnaryOperator::Call(args_value) => {
                    scope.create();
                    let args = args_value
                        .iter()
                        .map(|expr| eval_expr(expr, scope))
                        .collect::<Vec<Primitive>>();
                    if let Primitive::Function { args_name, code } = operand {
                        args_name
                            .into_iter()
                            .zip(args.into_iter())
                            .for_each(|(ident, value)| scope.add(ident, value));
                        let return_value = eval_expr(&code, scope);
                        scope.delete();
                        return_value
                    } else {
                        panic!("TypeError");
                    }
                }
                UnaryOperator::Index(index) => {
                    if let Primitive::Array(array) = operand
                        && let Primitive::Number(index) = eval_expr(index, scope)
                    {
                        array[index as usize].clone()
                    } else {
                        panic!("TypeError");
                    }
                }
            }
        }
        Expr::Binary {
            lhs: lhs_expr,
            operator,
            rhs: rhs_expr,
        } => {
            let lhs = eval_expr(lhs_expr, scope);
            let rhs = eval_expr(rhs_expr, scope);
            match operator {
                BinaryOperator::Add => {
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (lhs, rhs) {
                        Primitive::Number(lhs + rhs)
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Subtract => {
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (lhs, rhs) {
                        Primitive::Number(lhs - rhs)
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Multiply => {
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (lhs, rhs) {
                        Primitive::Number(lhs * rhs)
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Divide => {
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (lhs, rhs) {
                        Primitive::Number(lhs / rhs)
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Modulo => {
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (lhs, rhs) {
                        Primitive::Number(lhs % rhs)
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Assign => {
                    if let Expr::Literal(Literal::Ident(ident)) = &**lhs_expr {
                        scope.assign(ident, rhs);
                        Primitive::None
                    } else {
                        panic!("TypeError");
                    }
                }
                BinaryOperator::Equal => Primitive::Bool(
                    if let (Primitive::Number(lhs), Primitive::Number(rhs)) = (&lhs, &rhs) {
                        lhs == rhs
                    } else if let (Primitive::Bool(lhs), Primitive::Bool(rhs)) = (&lhs, &rhs) {
                        lhs == rhs
                    } else {
                        panic!("TypeError: これらの値は比較できません");
                    },
                ),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Primitive {
    Number(i32),
    Bool(bool),
    String(String),
    Array(Vec<Primitive>),
    Function { args_name: Vec<Ident>, code: Expr },
    None,
}

#[derive(Debug, Default)]
struct Scope {
    table: FxHashMap<Ident, Primitive>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    fn new() -> Self {
        Self {
            table: FxHashMap::default(),
            parent: None,
        }
    }
    fn create(&mut self) {
        let parent = mem::take(self);
        *self = Self {
            table: FxHashMap::default(),
            parent: Some(Box::new(parent)),
        };
    }
    fn add(&mut self, ident: Ident, value: Primitive) {
        self.table.insert(ident, value);
    }
    fn get(&self, ident: &Ident) -> Option<Primitive> {
        if let Some(primitive) = self.table.get(ident).cloned() {
            Some(primitive)
        } else {
            self.parent.as_ref()?.get(ident)
        }
    }
    fn get_mut(&mut self, ident: &Ident) -> Option<&mut Primitive> {
        if let Some(primitive) = self.table.get_mut(ident) {
            Some(primitive)
        } else {
            self.parent.as_mut()?.get_mut(ident)
        }
    }
    fn assign(&mut self, ident: &Ident, value: Primitive) {
        dbg!(&self);
        let var = self.get_mut(ident).unwrap();
        if mem::discriminant(var) == mem::discriminant(&value) {
            *var = value;
        } else {
            panic!("TypeError: Cannot assign");
        }
    }
    fn delete(&mut self) {
        let scopes = mem::take(self);
        *self = *scopes.parent.unwrap();
    }
}

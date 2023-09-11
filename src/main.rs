#![allow(unused)]

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    hash::Hash,
    io::{stdin, Read},
    rc::Rc,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct File {
    name: String,
    expression: Term,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Int {
    value: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bool {
    value: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Str {
    value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Print {
    value: Box<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Binary {
    rhs: Box<Term>,
    op: BinaryOp,
    lhs: Box<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Lt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct If {
    condition: Box<Term>,
    then: Box<Term>,
    otherwise: Box<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Let {
    name: Parameter,
    value: Box<Term>,
    next: Box<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Var {
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Function {
    parameters: Vec<Parameter>,
    value: Box<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Call {
    callee: Box<Term>,
    arguments: Vec<Term>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Term {
    Int(Int),
    Str(Str),
    Bool(Bool),
    Print(Print),
    Binary(Binary),
    If(If),
    Let(Let),
    Var(Var),
    Function(Function),
    Call(Call),
}

#[derive(Debug, Clone)]
pub enum Val {
    Void,
    Int(i32),
    Bool(bool),
    Str(String),
    Closure {
        body: Term,
        params: Vec<Parameter>,
        env: Rc<RefCell<Scope>>,
    },
}

pub type Scope = HashMap<String, Val>;

fn eval(term: Term, scope: &mut Scope) -> Val {
    match term {
        Term::Int(number) => Val::Int(number.value),
        Term::Str(str) => Val::Str(str.value),
        Term::Bool(bool) => Val::Bool(bool.value),
        Term::Print(print) => {
            let val = eval(*print.value, scope);
            match val {
                Val::Int(n) => print!("{n}"),
                Val::Bool(b) => print!("{b}"),
                Val::Str(s) => print!("{s}"),
                _ => panic!("valor não suportado"),
            };
            Val::Void
        }
        Term::Binary(bin) => {
            let lhs = eval(*bin.lhs, scope);
            let rhs = eval(*bin.rhs, scope);
            match bin.op {
                BinaryOp::Add => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Val::Int(a + b),
                    (Val::Str(a), Val::Int(b)) => Val::Str(format!("{a}{b}")),
                    (Val::Int(a), Val::Str(b)) => Val::Str(format!("{a}{b}")),
                    (Val::Str(a), Val::Str(b)) => Val::Str(format!("{a}{b}")),
                    _ => panic!("operadores inválidos"),
                },
                BinaryOp::Sub => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Val::Int(a - b),
                    _ => panic!("operadores inválidos"),
                },
                BinaryOp::Lt => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Val::Bool(a < b),
                    _ => panic!("operadores inválidos"),
                },
            }
        }

        Term::If(i) => match eval(*i.condition, scope) {
            Val::Bool(true) => eval(*i.then, scope),
            Val::Bool(false) => eval(*i.otherwise, scope),
            _ => panic!("condição inválida"),
        },
        Term::Let(l) => {
            let name = l.name.text;
            let mut value = match eval(*l.value, scope) {
                Val::Closure { body, params, env } => {
                    let closure = Val::Closure {
                        body,
                        params,
                        env: env.clone(),
                    };
                    env.borrow_mut().insert(name.clone(), closure.clone());
                    scope.insert(name.clone(), closure.clone());
                }
                val => {
                    scope.insert(name, val);
                }
            };
            eval(*l.next, scope)
        }
        Term::Var(v) => match scope.get(&v.text) {
            Some(val) => val.clone(),
            None => panic!("variável não encontrada"),
        },
        Term::Function(f) => Val::Closure {
            body: *f.value,
            params: f.parameters,
            env: Rc::new(RefCell::new(scope.clone())),
        },
        Term::Call(call) => match eval(*call.callee, scope) {
            Val::Closure { body, params, env } => {
                let mut new_scope = env.borrow_mut().clone();
                for (param, arg) in params.into_iter().zip(call.arguments) {
                    new_scope.insert(param.text, eval(arg, scope));
                }
                eval(body, &mut new_scope)
            }
            _ => panic!("não é uma função"),
        },
    }
}

fn main() {
    let mut program = String::new();
    stdin().lock().read_to_string(&mut program).unwrap();
    let program = serde_json::from_str::<File>(&program).expect("Não parseou");

    let term = program.expression;
    let mut scope = HashMap::new();
    eval(term, &mut scope);
}

#![allow(unused)]

use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    io::{stdin, Read},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct File {
    name: String,
    expression: Term,
}

#[derive(Debug, Deserialize)]
pub struct Int {
    value: i32,
}

#[derive(Debug, Deserialize)]
pub struct Bool {
    value: bool,
}

#[derive(Debug, Deserialize)]
pub struct Str {
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct Print {
    value: Box<Term>,
}

#[derive(Debug, Deserialize)]
pub struct Binary {
    rhs: Box<Term>,
    op: BinaryOp,
    lhs: Box<Term>,
}

#[derive(Debug, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
}

#[derive(Debug, Deserialize)]
pub struct If {
    condition: Box<Term>,
    then: Box<Term>,
    otherwise: Box<Term>,
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct Let {
    name: Parameter,
    value: Box<Term>,
    next: Box<Term>,
}

#[derive(Debug, Deserialize)]
pub struct Var {
    text: String,
}

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Clone)]
pub enum Val {
    Void,
    Int(i32),
    Bool(bool),
    Str(String),
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
        Term::Binary(bin) => match bin.op {
            BinaryOp::Add => {
                let lhs = eval(*bin.lhs, scope);
                let rhs = eval(*bin.rhs, scope);
                match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Val::Int(a + b),
                    (Val::Str(s), Val::Int(b)) => Val::Str(format!("{s}{b}")),
                    (Val::Int(s), Val::Str(b)) => Val::Str(format!("{s}{b}")),
                    (Val::Str(s), Val::Str(b)) => Val::Str(format!("{s}{b}")),
                    _ => panic!("operadores inválidos"),
                }
            }
            BinaryOp::Sub => {
                let lhs = eval(*bin.lhs, scope);
                let rhs = eval(*bin.rhs, scope);
                match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Val::Int(a - b),
                    _ => panic!("operadores inválidos"),
                }
            }
        },
        Term::If(i) => match eval(*i.condition, scope) {
            Val::Bool(true) => eval(*i.then, scope),
            Val::Bool(false) => eval(*i.otherwise, scope),
            _ => panic!("condição inválida"),
        },
        Term::Let(l) => {
            let name = l.name.text;
            let value = eval(*l.value, scope);
            scope.insert(name, value);
            eval(*l.next, scope)
        }
        Term::Var(v) => match scope.get(&v.text) {
            Some(val) => val.clone(),
            None => panic!("variável não encontrada"),
        },
    }
}

fn main() {
    let mut program = String::new();
    stdin().lock().read_to_string(&mut program).unwrap();
    let program = serde_json::from_str::<File>(&program).unwrap();

    let term = program.expression;
    let mut scope = HashMap::new();
    eval(term, &mut scope);
}

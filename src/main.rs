use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    io::{stdin, Read},
    rc::Rc,
};

use serde::Deserialize;

use crate::error::RuntimeError;

mod error;

#[derive(Debug, Deserialize)]
pub struct File {
    name: String,
    expression: Term,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Int {
    value: i32,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bool {
    value: bool,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Str {
    value: String,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Print {
    value: Box<Term>,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Binary {
    rhs: Box<Term>,
    op: BinaryOp,
    lhs: Box<Term>,
    location: Location,
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
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    text: String,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Let {
    name: Parameter,
    value: Box<Term>,
    next: Box<Term>,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Var {
    text: String,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Function {
    parameters: Vec<Parameter>,
    value: Box<Term>,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Call {
    callee: Box<Term>,
    arguments: Vec<Term>,
    location: Location,
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

impl Term {
    pub fn location(&self) -> &Location {
        match self {
            Term::Int(t) => &t.location,
            Term::Str(t) => &t.location,
            Term::Bool(t) => &t.location,
            Term::Print(t) => &t.location,
            Term::Binary(t) => &t.location,
            Term::If(t) => &t.location,
            Term::Let(t) => &t.location,
            Term::Var(t) => &t.location,
            Term::Function(t) => &t.location,
            Term::Call(t) => &t.location,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Void,
    Int(i32),
    Bool(bool),
    Str(String),
    Closure {
        fun: Function,
        env: Rc<RefCell<Scope>>,
    },
}

pub type Scope = HashMap<String, Val>;

fn eval(term: Term, scope: &mut Scope) -> Result<Val, RuntimeError> {
    match term {
        Term::Int(number) => Ok(Val::Int(number.value)),
        Term::Str(str) => Ok(Val::Str(str.value)),
        Term::Bool(bool) => Ok(Val::Bool(bool.value)),
        Term::Print(print) => {
            let val = eval(*print.value, scope)?;
            match val {
                Val::Int(n) => print!("{n}"),
                Val::Bool(b) => print!("{b}"),
                Val::Str(s) => print!("{s}"),
                Val::Closure { .. } => print!("<#closure>"),
                Val::Void => {}
            };
            Ok(Val::Void)
        }

        Term::Binary(bin) => {
            let lhs = eval(*bin.lhs, scope)?;
            let rhs = eval(*bin.rhs, scope)?;
            match bin.op {
                BinaryOp::Add => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Int(a + b)),
                    (Val::Str(a), Val::Int(b)) => Ok(Val::Str(format!("{a}{b}"))),
                    (Val::Int(a), Val::Str(b)) => Ok(Val::Str(format!("{a}{b}"))),
                    (Val::Str(a), Val::Str(b)) => Ok(Val::Str(format!("{a}{b}"))),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
                BinaryOp::Sub => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Int(a - b)),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
                BinaryOp::Lt => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a < b)),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
            }
        }

        Term::If(i) => {
            let location = i.condition.location().clone();
            match eval(*i.condition, scope)? {
                Val::Bool(true) => eval(*i.then, scope),
                Val::Bool(false) => eval(*i.otherwise, scope),
                _ => Err(RuntimeError::new("condição inválida", location)),
            }
        }

        Term::Let(l) => {
            let name = l.name.text;
            match eval(*l.value, scope)? {
                Val::Closure { fun, env } => {
                    let closure = Val::Closure {
                        fun,
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
            Some(val) => Ok(val.clone()),
            None => Err(RuntimeError::unknow_identifier(v)),
        },

        Term::Function(fun) => Ok(Val::Closure {
            fun,
            env: Rc::new(RefCell::new(scope.clone())),
        }),

        Term::Call(call) => match eval(*call.callee.clone(), scope)? {
            Val::Closure { fun, env } => {
                if call.arguments.len() != fun.parameters.len() {
                    return Err(RuntimeError::invalid_number_of_arguments(fun, call));
                }

                let mut new_scope = env.borrow_mut().clone();
                for (param, arg) in fun.parameters.into_iter().zip(call.arguments) {
                    new_scope.insert(param.text, eval(arg, scope)?);
                }
                eval(*fun.value, &mut new_scope)
            }
            _ => Err(RuntimeError::new("não é uma função", call.location)),
        },
    }
}

fn main() {
    let mut program = String::new();
    stdin().lock().read_to_string(&mut program).unwrap();
    let program = serde_json::from_str::<File>(&program).expect("Não parseou");

    let term = program.expression;
    let mut scope = HashMap::new();
    if let Err(error) = eval(term, &mut scope) {
        if let Ok(source) = fs::read_to_string(program.name) {
            let report = miette::Report::new(error).with_source_code(source);
            print!("{:?}", report)
        } else {
            println!("{}", error);
        }
    }
}

use std::{
    cell::RefCell,
    collections::HashMap,
    env::args,
    fmt::Display,
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
    value: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Binary {
    rhs: Term,
    op: BinaryOp,
    lhs: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, Deserialize)]
pub struct If {
    condition: Term,
    then: Term,
    otherwise: Term,
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
    value: Term,
    next: Term,
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
    value: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Call {
    callee: Term,
    arguments: Vec<Term>,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tuple {
    first: Term,
    second: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct First {
    value: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Second {
    value: Term,
    location: Location,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Term {
    Int(Int),
    Str(Str),
    Bool(Bool),
    Print(Box<Print>),
    Binary(Box<Binary>),
    If(Box<If>),
    Let(Box<Let>),
    Var(Var),
    Function(Box<Function>),
    Call(Box<Call>),
    Tuple(Box<Tuple>),
    First(Box<First>),
    Second(Box<Second>),
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
            Term::Tuple(t) => &t.location,
            Term::First(t) => &t.location,
            Term::Second(t) => &t.location,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Int(i32),
    Bool(bool),
    Str(String),
    Tuple((Box<Val>, Box<Val>)),
    Closure { fun: Function, env: Scope },
}

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => a == b,
            (Val::Bool(a), Val::Bool(b)) => a == b,
            (Val::Str(a), Val::Str(b)) => a == b,
            (Val::Tuple(a), Val::Tuple(b)) => a == b,
            _ => false,
        }
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Int(i) => write!(f, "{i}"),
            Val::Bool(true) => write!(f, "true"),
            Val::Bool(false) => write!(f, "false"),
            Val::Str(s) => write!(f, "{s}"),
            Val::Tuple((fst, snd)) => write!(f, "({fst}, {snd})"),
            Val::Closure { .. } => write!(f, "<#closure>"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    current: Rc<RefCell<HashMap<String, Val>>>,
}

impl Scope {
    pub fn get(&self, var: &str) -> Option<Val> {
        self.current
            .borrow()
            .get(var)
            .cloned()
            .or_else(|| self.parent.as_ref()?.get(var))
    }

    pub fn set(&self, var: impl Into<String>, val: Val) {
        self.current.borrow_mut().insert(var.into(), val);
    }
}

impl Clone for Scope {
    fn clone(&self) -> Self {
        Scope {
            parent: Some(Rc::new(Scope {
                parent: self.parent.clone(),
                current: self.current.clone(),
            })),
            current: Default::default(),
        }
    }
}

fn eval(term: Term, scope: &Scope) -> Result<Val, RuntimeError> {
    match term {
        Term::Int(number) => Ok(Val::Int(number.value)),
        Term::Str(str) => Ok(Val::Str(str.value)),
        Term::Bool(bool) => Ok(Val::Bool(bool.value)),
        Term::Print(print) => {
            let val = eval(print.value, scope)?;
            println!("{val}");
            Ok(val)
        }
        Term::Tuple(tuple) => Ok(Val::Tuple((
            Box::new(eval(tuple.first, scope)?),
            Box::new(eval(tuple.second, scope)?),
        ))),
        Term::First(t) => match eval(t.value, scope)? {
            Val::Tuple((val, _)) => Ok(*val),
            _ => Err(RuntimeError::new("não é uma tupla", t.location)),
        },
        Term::Second(t) => match eval(t.value, scope)? {
            Val::Tuple((_, val)) => Ok(*val),
            _ => Err(RuntimeError::new("não é uma tupla", t.location)),
        },

        Term::Binary(bin) => {
            let lhs = eval(bin.lhs, scope)?;
            let rhs = eval(bin.rhs, scope)?;

            macro_rules! bin_op {
                ($left:ident[$lhs:expr], $right:ident[$rhs:expr] -> $f:expr) => {
                    match (lhs, rhs) {
                        (Val::$left(lhs), Val::$right(rhs)) => $f(lhs, rhs),
                        _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                    }
                };
            }
            #[allow(clippy::redundant_closure_call)]
            match bin.op {
                BinaryOp::Add => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Int(a + b)),
                    (a, b) => Ok(Val::Str(format!("{a}{b}"))),
                },
                BinaryOp::Sub => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Int(a - b))),
                BinaryOp::Mul => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Int(a * b))),
                BinaryOp::Div => match (lhs, rhs) {
                    (Val::Int(_), Val::Int(0)) => Err(RuntimeError::division_by_zero(bin.location)),
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Int(a / b)),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
                BinaryOp::Rem => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Int(a % b))),
                BinaryOp::And => bin_op!(Bool[lhs], Bool[rhs] -> |a, b| Ok(Val::Bool(a && b))),
                BinaryOp::Or => bin_op!(Bool[lhs], Bool[rhs] -> |a, b| Ok(Val::Bool(a || b))),
                BinaryOp::Lt => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Bool(a < b))),
                BinaryOp::Lte => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Bool(a <= b))),
                BinaryOp::Gt => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Bool(a > b))),
                BinaryOp::Gte => bin_op!(Int[lhs], Int[rhs] -> |a, b| Ok(Val::Bool(a >= b))),
                BinaryOp::Eq => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a == b)),
                    (Val::Bool(a), Val::Bool(b)) => Ok(Val::Bool(a == b)),
                    (Val::Str(a), Val::Str(b)) => Ok(Val::Bool(a == b)),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
                BinaryOp::Neq => match (lhs, rhs) {
                    (Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a != b)),
                    (Val::Bool(a), Val::Bool(b)) => Ok(Val::Bool(a != b)),
                    (Val::Str(a), Val::Str(b)) => Ok(Val::Bool(a != b)),
                    _ => Err(RuntimeError::invalid_binary_operation(bin.location)),
                },
            }
        }

        Term::If(i) => {
            let location = i.condition.location().clone();
            match eval(i.condition, scope)? {
                Val::Bool(true) => eval(i.then, scope),
                Val::Bool(false) => eval(i.otherwise, scope),
                _ => Err(RuntimeError::new("condição inválida", location)),
            }
        }

        Term::Let(l) => {
            let name = l.name.text;
            scope.set(name, eval(l.value, scope)?);
            eval(l.next, scope)
        }

        Term::Var(v) => match scope.get(&v.text) {
            Some(val) => Ok(val.clone()),
            None => Err(RuntimeError::unknow_identifier(v)),
        },

        Term::Function(fun) => Ok(Val::Closure {
            fun: *fun,
            env: scope.clone(),
        }),

        Term::Call(call) => match eval(call.callee, scope)? {
            Val::Closure { fun, env } => {
                if call.arguments.len() != fun.parameters.len() {
                    return Err(RuntimeError::invalid_number_of_arguments(
                        fun,
                        call.location,
                    ));
                }

                for (param, arg) in fun.parameters.into_iter().zip(call.arguments) {
                    env.set(param.text, eval(arg, scope)?);
                }

                eval(fun.value, &env)
            }
            _ => Err(RuntimeError::new("não é uma função", call.location)),
        },
    }
}

fn main() {
    let program = match args().nth(1) {
        Some(file) => fs::read_to_string(file).expect("Arquivo não encontrado"),
        None => {
            let mut buf = String::new();
            stdin().lock().read_to_string(&mut buf).unwrap();
            buf
        }
    };

    let program = {
        let mut deserializer = serde_json::Deserializer::from_str(&program);
        deserializer.disable_recursion_limit();
        let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
        File::deserialize(deserializer).expect("Programa inválido")
    };

    let term = program.expression;
    let scope = Scope::default();
    if let Err(error) = eval(term, &scope) {
        if let Ok(source) = fs::read_to_string(program.name) {
            let report = miette::Report::new(error).with_source_code(source);
            print!("{:?}", report)
        } else {
            println!("{}", error);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_test() {
        let s1 = Scope::default();
        s1.set("a", Val::Int(1));
        s1.set("b", Val::Int(2));

        let s2 = s1.clone();
        assert_eq!(s1.get("a"), Some(Val::Int(1)));
        assert_eq!(s2.get("a"), Some(Val::Int(1)));
        s2.set("a", Val::Int(2));
        assert_eq!(s2.get("a"), Some(Val::Int(2)));

        let s3 = s2.clone();
        assert_eq!(s3.get("a"), Some(Val::Int(2)));
        assert_eq!(s3.get("b"), Some(Val::Int(2)));
    }
}

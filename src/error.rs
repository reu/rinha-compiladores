use std::fmt::Display;

use miette::{Diagnostic, LabeledSpan};

use crate::{Function, Location, Var};

#[derive(Debug)]
pub enum ErrorKind {
    ArgumentError,
    DivisionByZero,
    UnknowIdentifier(Var),
    InvalidBinaryOperation,
    InvalidNumberOfArguments(Function, Location),
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    location: Location,
    kind: ErrorKind,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>, location: Location) -> Self {
        Self {
            message: message.into(),
            location,
            kind: ErrorKind::ArgumentError,
        }
    }

    pub fn unknow_identifier(var: Var) -> Self {
        Self {
            message: "identificador não encontrado".into(),
            location: var.location.clone(),
            kind: ErrorKind::UnknowIdentifier(var),
        }
    }

    pub fn division_by_zero(loc: Location) -> Self {
        Self {
            message: "divisão por zero".into(),
            location: loc,
            kind: ErrorKind::DivisionByZero,
        }
    }

    pub fn invalid_binary_operation(loc: Location) -> Self {
        Self {
            message: "operação inválida".into(),
            location: loc,
            kind: ErrorKind::InvalidBinaryOperation,
        }
    }

    pub fn invalid_number_of_arguments(fun: Function, loc: Location) -> Self {
        Self {
            message: "número de argumentos inválidos".into(),
            location: loc.clone(),
            kind: ErrorKind::InvalidNumberOfArguments(fun, loc),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl Diagnostic for RuntimeError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match self.kind {
            ErrorKind::ArgumentError
            | ErrorKind::InvalidBinaryOperation
            | ErrorKind::DivisionByZero => Some(Box::new(
                [LabeledSpan::at(
                    self.location.start..self.location.end,
                    self.message.clone(),
                )]
                .into_iter(),
            )),

            ErrorKind::UnknowIdentifier(ref var) => Some(Box::new(
                [LabeledSpan::at(
                    var.location.start..var.location.end,
                    self.message.clone(),
                )]
                .into_iter(),
            )),

            ErrorKind::InvalidNumberOfArguments(ref fun, ref loc) => Some(Box::new(
                [
                    LabeledSpan::at(loc.start..loc.end, "parâmetros informados"),
                    LabeledSpan::at(
                        if fun.parameters.is_empty() {
                            fun.location.start..fun.location.start + 2
                        } else {
                            let first_param = fun.parameters.first().unwrap();
                            let last_param = fun.parameters.last().unwrap();
                            first_param.location.start..last_param.location.end
                        },
                        "argumentos esperados",
                    ),
                ]
                .into_iter(),
            )),
        }
    }
}

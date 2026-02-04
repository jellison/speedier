use super::expression::{parse_expression, Expr};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct EvaluatorError {
    message: String,
}

impl EvaluatorError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EvaluatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EvaluatorError {}

pub struct Evaluator {
    last_result: f64,
}

impl Evaluator {
    pub fn with_last(last_result: f64) -> Self {
        Self { last_result }
    }

    pub fn eval(&mut self, expr: &str) -> Result<f64, EvaluatorError> {
        let ast = parse_expression(expr)?;
        let result = self.eval_expr(&ast)?;
        self.last_result = result;
        Ok(result)
    }

    pub fn last_result(&self) -> f64 {
        self.last_result
    }

    fn eval_expr(&self, expr: &Expr) -> Result<f64, EvaluatorError> {
        match expr {
            Expr::Number(value) => Ok(*value),
            Expr::Ident(name) => self.eval_ident(name),
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                match op {
                    '+' => Ok(val),
                    '-' => Ok(-val),
                    _ => Err(EvaluatorError::new(format!(
                        "unsupported unary operator '{}'",
                        op
                    ))),
                }
            }
            Expr::Binary { op, left, right } => {
                let lhs = self.eval_expr(left)?;
                let rhs = self.eval_expr(right)?;
                match op {
                    '+' => Ok(lhs + rhs),
                    '-' => Ok(lhs - rhs),
                    '*' => Ok(lhs * rhs),
                    '/' => Ok(lhs / rhs),
                    '%' => Ok(lhs % rhs),
                    '^' => Ok(lhs.powf(rhs)),
                    _ => Err(EvaluatorError::new(format!(
                        "unsupported operator '{}'",
                        op
                    ))),
                }
            }
            Expr::Call { name, args } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval_expr(arg)?);
                }
                self.eval_call(name, &values)
            }
        }
    }

    fn eval_ident(&self, name: &str) -> Result<f64, EvaluatorError> {
        match name.to_lowercase().as_str() {
            "ans" => Ok(self.last_result),
            "pi" => Ok(std::f64::consts::PI),
            "e" => Ok(std::f64::consts::E),
            "nan" => Ok(f64::NAN),
            "inf" | "infinity" => Ok(f64::INFINITY),
            _ => Err(EvaluatorError::new(format!(
                "unknown identifier '{}'",
                name
            ))),
        }
    }

    fn eval_call(&self, name: &str, args: &[f64]) -> Result<f64, EvaluatorError> {
        match name.to_lowercase().as_str() {
            "sin" => Ok(args.get(0).copied().unwrap_or(0.0).sin()),
            "cos" => Ok(args.get(0).copied().unwrap_or(0.0).cos()),
            "tan" => Ok(args.get(0).copied().unwrap_or(0.0).tan()),
            "sqrt" => Ok(args.get(0).copied().unwrap_or(0.0).sqrt()),
            "pow" => {
                if args.len() < 2 {
                    return Err(EvaluatorError::new("pow expects two arguments"));
                }
                Ok(args[0].powf(args[1]))
            }
            "log" => Ok(args.get(0).copied().unwrap_or(0.0).log10()),
            "ln" => Ok(args.get(0).copied().unwrap_or(0.0).ln()),
            "abs" => Ok(args.get(0).copied().unwrap_or(0.0).abs()),
            "ceil" => Ok(args.get(0).copied().unwrap_or(0.0).ceil()),
            "floor" => Ok(args.get(0).copied().unwrap_or(0.0).floor()),
            _ => Err(EvaluatorError::new(format!(
                "unknown function '{}'",
                name
            ))),
        }
    }
}

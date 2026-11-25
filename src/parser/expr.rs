use std::{collections::HashMap, f64};

use crate::parser::function::Function;

#[derive(Debug, Clone)]
pub enum Expr {
	Add(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
	Unary(Box<Expr>),
	Exponent(Box<Expr>, Box<Expr>),
	FunctionCall { name: char, params: Vec<Expr> },
	Log { base: Box<Expr>, value: Box<Expr> },
	Sine(Box<Expr>),
	Cosine(Box<Expr>),
	Tangent(Box<Expr>),
	Asine(Box<Expr>),
	Acosine(Box<Expr>),
	Atangent(Box<Expr>),
	VariableRef(char),
	Literal(f64),
	Pi,
	EulersNumber,
}

impl Expr {
	pub fn execute(
		self,
		variables: &HashMap<char, Expr>,
		functions: &HashMap<char, Function>,
	) -> f64 {
		return match self {
			Expr::Add(lhs, rhs) =>
				lhs.execute(variables, functions) + rhs.execute(variables, functions),
			Expr::Sub(lhs, rhs) =>
				lhs.execute(variables, functions) - rhs.execute(variables, functions),
			Expr::Mul(lhs, rhs) =>
				lhs.execute(variables, functions) * rhs.execute(variables, functions),
			Expr::Div(lhs, rhs) =>
				lhs.execute(variables, functions) / rhs.execute(variables, functions),
			Expr::Unary(expr) => -expr.execute(variables, functions),
			Expr::Exponent(lhs, rhs) => lhs
				.execute(variables, functions)
				.powf(rhs.execute(variables, functions)),
			Expr::FunctionCall { name, params } => {
				let function = functions.get(&name).unwrap().clone();
				function.run(params)
			},
			Expr::Log { base, value } => value
				.execute(variables, functions)
				.log(base.execute(variables, functions)),
			Expr::Sine(expr) => expr.execute(variables, functions).sin(),
			Expr::Cosine(expr) => expr.execute(variables, functions).cos(),
			Expr::Tangent(expr) => expr.execute(variables, functions).tan(),
			Expr::Asine(expr) => expr.execute(variables, functions).asin(),
			Expr::Acosine(expr) => expr.execute(variables, functions).acos(),
			Expr::Atangent(expr) => expr.execute(variables, functions).atan(),
			Expr::VariableRef(character) => variables
				.get(&character)
				.unwrap()
				.clone()
				.execute(variables, functions),
			Expr::Literal(value) => value,
			Expr::Pi => f64::consts::PI,
			Expr::EulersNumber => f64::consts::E,
		};
	}
}

pub mod expr;
pub mod function;

use expr::Expr::{self, *};
use crate::parser::ParserError::*;

use std::{
	char,
	error::Error, fmt::{Display, Formatter, Result as FmtResult},
	str::Chars,
};

#[must_use]
#[derive(Debug)]
pub enum ParserError {
	ExpectedMoreChars,
	UnexpectedChar(char),
	UnexpectedIdentifier(String),
	UnmatchedBracket,
	MultipleDecimalPLaces,
	LogBaseUndecleared,
}

impl Error for ParserError {}

impl Display for ParserError {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::ExpectedMoreChars => write!(fmt, "More Characters Expected!"),
			Self::UnexpectedChar(character) => write!(fmt, "Unexpected Character: {character:?}"),
			Self::UnexpectedIdentifier(identifier) => write!(fmt, "Unexpected Identifier: {identifier:?}"),
			Self::UnmatchedBracket => write!(fmt, "Unmatched Bracket!"),
			Self::MultipleDecimalPLaces => write!(fmt, "Multiple Decimal Places For One Number!"),
			Self::LogBaseUndecleared => write!(fmt, "Log Base Undecleared!"),
		}
	}
}

#[derive(Debug)]
struct Source<'a> {
	input: Chars<'a>,
	peeked: Option<char>,
}

impl<'a> From<&'a str> for Source<'a> {
	fn from(value: &'a str) -> Self {
		Source {
			input: value.chars(),
			peeked: None,
		}
	}
}
impl <'a> From<&'a String> for Source<'a> {
	fn from(value: &'a String) -> Self {
		Source { 
			input: value.chars(),
			peeked: None,
		}
	}
	
}

impl Iterator for Source<'_> {
	type Item = char;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self.peeked {
			None => {
				loop {
					let next: char = self.input.next()?;
					if next.is_whitespace() {
						continue;
					};
					return Some(next)
				}
			},
			_ => self.peeked.take(),
		}
	}
}
	
impl Source<'_> {
    #[inline]
    pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        if let None = self.peeked {
            self.peeked = self.next();
        };
        self.peeked.as_ref()
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    source: Source<'a>
}

impl<'a> Parser<'a> {
	pub fn new(source: &'a str) -> Self {
		Parser { source: Source::from(source) }
	}

	pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
		self.parse_term_expr()
	}

	fn parse_term_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_factor_expr()?;
		let mut next: char;

		'outer: loop {
			let peeked = self.source.peek();
			match peeked {
				None => break 'outer,
				Some(value) => next = *value
			}

			match next {
				'+' => {
					self.source.next();
					result = Add(Box::new(result), Box::new(self.parse_factor_expr()?));
				},
				'-' => {
					self.source.next();
					result = Sub(Box::new(result), Box::new(self.parse_factor_expr()?));
				},
				_ => break 'outer
			}
		}
		Ok(result)
	}

	fn parse_factor_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_unary_expr()?;
		let mut next: char;

		'outer: loop {
			let peeked: Option<&char> = self.source.peek();
			match peeked {
				None => break 'outer,
				Some(value) => next = *value
			}

			match next {
				'*' => {
					self.source.next();
					result = Mul(Box::new(result), Box::new(self.parse_unary_expr()?));
				},
				'/' => {
					self.source.next();
					result = Div(Box::new(result), Box::new(self.parse_unary_expr()?));
				},
				_ => break 'outer
			}
		}
		Ok(result)
	}

	fn parse_unary_expr(&mut self) -> Result<Expr, ParserError> {
		if self.source.peek().filter(|c: &&char| **c == '-').is_some() {
			self.source.next();
			return Ok(Unary(Box::new(self.parse_exponent_expr()?)))
		}
		self.parse_exponent_expr()
	}

	fn parse_exponent_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_function_expr()?;
		let mut next: char;

		'outer: loop {
			let peeked: Option<&char> = self.source.peek();
			match peeked {
				None => break 'outer,
				Some(value) => next = *value
			}

			match next {
				'^' => {
					self.source.next();
					result = Exponent(Box::new(result), Box::new(self.parse_function_expr()?));
				},
				_ => break 'outer
			}
		}
		Ok(result)
	}

	fn parse_function_expr(&mut self) -> Result<Expr, ParserError> {
		let peeked: Option<&char> = self.source.peek();
		let next: char;
		match peeked {
			None => return Err(ExpectedMoreChars),
			Some(value) => next = *value
		}

		if !next.is_alphabetic() {
			return Ok(self.parse_primery_expr()?)
		} else {
			self.source.next();
		}

		if let Some('(') = self.source.peek() {
			self.source.next();
			let mut parameters: Vec<Expr> = Vec::new();
			'outer: loop {
				parameters.push(self.parse_expr()?);
				let read: char = self.source.next().unwrap();
				match read {
					',' => continue,
					')' => break 'outer,
					_ => return Err(UnexpectedChar(read)),
				}
			}
			return Ok(FunctionCall { 
				name: next,
				params: parameters
			})

		} else if let 'a'|'l' | 's' | 'c' | 't' | 'p' | 'e' = next {
			if next == 'e' {
				return Ok(EulersNumber)
			}

			let mut identifier: String = String::with_capacity(4);
			identifier.push(next);
			let peeked: Option<&char> = self.source.peek();
			match peeked {
				None => return Ok(VariableRef(next)),
				Some(value) => {
					match value {
						'n'|'o'|'i'|'a' => {
							identifier.push(*value);
							self.source.next();
						},
						_ => return Ok(VariableRef(next))
					}
				},
			}

			let peeked: Option<&char> = self.source.peek();
			match identifier.as_str() {
				"pi" => return Ok(Pi),
				"ln" => {
					let next: char = *peeked.unwrap();
					if next != '(' {
						return Err(UnexpectedChar(next))
					}
					self.source.next();
					
					let expr: Expr = self.parse_expr()?;
					if self.source.next() != Some(')') {
						return Err(UnmatchedBracket)
					}
					return Ok(Log { base: Box::new(EulersNumber), value: Box::new(expr) })
				},
				_ => {
					identifier.push(*peeked.unwrap());
					self.source.next();
				}				
			}

			let next: char = self.source.next().unwrap();
			let mut base: Result<Expr, ParserError> = Err(LogBaseUndecleared);
			match next {
				'_' => {
					base = self.parse_expr();
					self.source.next();
					let next: Option<char> = self.source.next();
					if next != Some('(') {
						return Err(UnexpectedChar(next.unwrap()))
					}
				},
				'n' | 's' => {
					identifier.push(next);
					let next: char = self.source.next().unwrap();
					if next != '(' {
						return Err(UnexpectedChar(next))
					}
				}
				_ => ()
			}

			let expr: Expr = self.parse_expr()?;
			let Some(')'): Option<char> = self.source.next() else {
  				return Err(UnmatchedBracket);
			};

			match identifier.as_str() {
				"sin" => return Ok(Sine(Box::new(expr))),
				"cos" => return Ok(Cosine(Box::new(expr))),
				"tan" => return Ok(Tangent(Box::new(expr))),
				"asin" => return Ok(Asine(Box::new(expr))),
				"acos" => return Ok(Acosine(Box::new(expr))),
				"atan" => return Ok(Atangent(Box::new(expr))),
				"log" => return Ok(Log { base: Box::new(base?), value: Box::new(expr) }),
				_ => return Err(UnexpectedIdentifier(identifier)),
			}


			// let name: Option<&char> = self.source.peek();
			// if name.is_none() && *name.unwrap() != 'i' && *name.unwrap() != 'o' && *name.unwrap() != 'a' {
			// 	return Ok(self.parse_primery_expr()?)
			// }
			
			// let name: char = *name.unwrap();
			// self.source.next();
			// let end: char = self.source.next().unwrap();
			// let full: String = format!("{}{}{}", next, name, end);	

			// let mut character: char = self.source.next().unwrap();
			// let base: Result<Expr, ParserError>;
			// if character == '_' {
			// 	base = self.parse_expr();
			// } else {
			// 	base = Err(LogBaseUndecleared);
			// }

			// if character != '(' {
			// 	return Err(UnexpectedChar(character))
			// }
			// let expr: Expr = self.parse_expr()?;	

			// character = self.source.next().unwrap();
			// if character != ')' {
			// 	return Err(UnexpectedChar(character));
			// }

			// return match full.as_str() {
			// 	"sin" => Ok(Sine(Box::new(expr))),
			// 	"cos" => Ok(Cosine(Box::new(expr))),
			// 	"tan" => Ok(Tangent(Box::new(expr))),
			// 	"log" => {
			// 		Ok(Log { base: Box::new(base?), value: Box::new(expr) })
			// 	},
			// 	_ => Err(UnexpectedIdentifier(full)),
			// }
		}
		Ok(VariableRef(next))
	}

	fn parse_primery_expr(&mut self) -> Result<Expr, ParserError> {
		let peeked: Option<&char> = self.source.peek();
		if peeked.is_none() {
			return Err(ExpectedMoreChars)
		}

		if *peeked.unwrap() == '(' {
			self.source.next();
			let expr: Result<Expr, ParserError> = self.parse_expr();

			if self.source.next().filter(|c| *c == ')').is_none() {
				return Err(UnmatchedBracket)
			}
			return Ok(expr?)
		} else if peeked.unwrap().is_numeric() {
			Ok(Literal(self.parse_number()?))
		} else {
			return Err(UnexpectedChar(*peeked.unwrap()))
		}
	}

	fn parse_number(&mut self) -> Result<f64, ParserError> {
		let mut buf: String = "".to_string();
		let mut seen_dot: bool = false;
		'outer: loop {
			let peeked: Option<&char> = self.source.peek();
			if peeked.is_none() {
				break 'outer;
			}
			let next: char = *peeked.unwrap();
			if next.is_numeric() {
				self.source.next();
			} else if next == '.' {
				self.source.next();
				if seen_dot {
					return Err(MultipleDecimalPLaces)
				}
				seen_dot = true;
			} else {
				break 'outer;
			}
			buf.push(next);
		}
		Ok(buf.parse::<f64>().unwrap())
	}
}
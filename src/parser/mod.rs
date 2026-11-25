use expr::Expr::{self, *};

pub mod expr;
pub mod function;

use std::{
	char,
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
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
			Self::UnexpectedIdentifier(identifier) => {
				write!(fmt, "Unexpected Identifier: {identifier:?}")
			},
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
		Self {
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
			None => loop {
				let next: char = self.input.next()?;
				if next.is_whitespace() {
					continue;
				};
				return Some(next);
			},
			_ => self.peeked.take(),
		}
	}
}

impl Source<'_> {
	#[inline]
	#[must_use]
	pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
		if let None = self.peeked {
			self.peeked = self.next();
		};
		self.peeked.as_ref()
	}
}

#[derive(Debug)]
pub struct Parser<'a> {
	source: Source<'a>,
}

impl<'a> Parser<'a> {
	pub fn new(source: &'a str) -> Self {
		Self {
			source: Source::from(source),
		}
	}

	pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
		self.parse_term_expr()
	}

	fn parse_term_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_factor_expr()?;
		let mut next: char;

		loop {
			match self.source.peek() {
				None => break,
				Some(value) => next = *value,
			};

			match next {
				'+' => {
					self.source.next();
					result = Add(Box::new(result), Box::new(self.parse_factor_expr()?));
				},
				'-' => {
					self.source.next();
					result = Sub(Box::new(result), Box::new(self.parse_factor_expr()?));
				},
				_ => break,
			};
		}
		Ok(result)
	}

	fn parse_factor_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_unary_expr()?;
		let mut next: char;

		loop {
			match self.source.peek() {
				None => break,
				Some(value) => next = *value,
			};

			match next {
				'*' => {
					self.source.next();
					result = Mul(Box::new(result), Box::new(self.parse_unary_expr()?));
				},
				'/' => {
					self.source.next();
					result = Div(Box::new(result), Box::new(self.parse_unary_expr()?));
				},
				_ => break,
			};
		}
		Ok(result)
	}

	fn parse_unary_expr(&mut self) -> Result<Expr, ParserError> {
		if let Some('-') = self.source.peek() {
			self.source.next();
			return Ok(Unary(Box::new(self.parse_exponent_expr()?)));
		};
		self.parse_exponent_expr()
	}

	fn parse_exponent_expr(&mut self) -> Result<Expr, ParserError> {
		let mut result: Expr = self.parse_function_expr()?;
		let mut next: char;

		loop {
			match self.source.peek() {
				None => break,
				Some(value) => next = *value,
			};
			match next {
				'^' => {
					self.source.next();
					result = Exponent(Box::new(result), Box::new(self.parse_function_expr()?));
				},
				_ => break,
			};
		}
		Ok(result)
	}

	fn parse_function_expr(&mut self) -> Result<Expr, ParserError> {
		let next: char;
		match self.source.peek() {
			None => return Err(ParserError::ExpectedMoreChars),
			Some(value) => next = *value,
		};

		if !next.is_alphabetic() {
			return Ok(self.parse_primery_expr()?);
		} else {
			let _ = self.source.next();
		};

		if let Some('(') = self.source.peek() {
			let _ = self.source.next();
			let mut parameters: Vec<Expr> = Vec::new();
			loop {
				parameters.push(self.parse_expr()?);
				match self.source.next().unwrap() {
					',' => continue,
					')' => break,
					unexpected => return Err(ParserError::UnexpectedChar(unexpected)),
				};
			}
			return Ok(FunctionCall {
				name: next,
				params: parameters,
			});
		} else if let 'a' | 'l' | 's' | 'c' | 't' | 'p' | 'e' = next {
			if next == 'e' {
				return Ok(EulersNumber);
			};

			let mut identifier: String = String::with_capacity(4);
			identifier.push(next);
			let peeked: Option<&char> = self.source.peek();
			match peeked {
				None => return Ok(VariableRef(next)),
				Some(value) => match value {
					'n' | 'o' | 'i' | 'a' => {
						identifier.push(*value);
						let _ = self.source.next();
					},
					_ => return Ok(VariableRef(next)),
				},
			};

			match identifier.as_str() {
				"pi" => return Ok(Pi),
				"ln" => {
					let Some('('): Option<char> = self.source.next() else {
						return Err(ParserError::UnexpectedChar(next));
					};

					let expr: Expr = self.parse_expr()?;
					let Some(')'): Option<char> = self.source.next() else {
						return Err(ParserError::UnmatchedBracket);
					};
					return Ok(Log {
						base: Box::new(EulersNumber),
						value: Box::new(expr),
					});
				},
				_ => identifier.push(self.source.next().unwrap()),
			}

			let next: char = self.source.next().unwrap();
			let mut base: Result<Expr, ParserError> = Err(ParserError::LogBaseUndecleared);
			match next {
				'_' => {
					base = self.parse_expr();
					let _ = self.source.next();
					let Some('('): Option<char> = self.source.next() else {
						return Err(ParserError::UnexpectedChar(next));
					};
				},
				'n' | 's' => {
					identifier.push(next);
					let next: char = self.source.next().unwrap();
					if next != '(' {
						return Err(ParserError::UnexpectedChar(next));
					};
				},
				_ => (),
			};

			let expr: Expr = self.parse_expr()?;
			let Some(')'): Option<char> = self.source.next() else {
				return Err(ParserError::UnmatchedBracket);
			};

			match identifier.as_str() {
				"sin" => return Ok(Sine(Box::new(expr))),
				"cos" => return Ok(Cosine(Box::new(expr))),
				"tan" => return Ok(Tangent(Box::new(expr))),
				"asin" => return Ok(Asine(Box::new(expr))),
				"acos" => return Ok(Acosine(Box::new(expr))),
				"atan" => return Ok(Atangent(Box::new(expr))),
				"log" => {
					return Ok(Log {
						base: Box::new(base?),
						value: Box::new(expr),
					});
				},
				_ => return Err(ParserError::UnexpectedIdentifier(identifier)),
			};

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
		let Some(next) = self.source.peek() else {
			return Err(ParserError::ExpectedMoreChars);
		};

		if *next == '(' {
			let _ = self.source.next();
			let expr: Expr = self.parse_expr()?;

			let Some(')'): Option<char> = self.source.next() else {
				return Err(ParserError::UnmatchedBracket);
			};
			Ok(expr)
		} else if next.is_numeric() {
			Ok(Literal(self.parse_number()?))
		} else {
			Err(ParserError::UnexpectedChar(*next))
		}
	}

	fn parse_number(&mut self) -> Result<f64, ParserError> {
		let mut buf: String = String::with_capacity(14);
		let mut seen_dot: bool = false;
		loop {
			let Some(next): Option<&char> = self.source.peek() else {
				break;
			};
			// SANITY(unusual): Borrowing rules.
			let next: char = *next;
			match next {
				'0'..'9' => {
					let _ = self.source.next();
				},
				'.' => {
					let _ = self.source.next();
					if seen_dot {
						return Err(ParserError::MultipleDecimalPLaces);
					};
					seen_dot = true;
				},
				_ => break,
			};
			buf.push(next);
		}
		// FIXME: Add an error enumeration for this.
		Ok(buf.parse::<f64>().unwrap())
	}
}

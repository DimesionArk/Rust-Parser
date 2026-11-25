use std::collections::HashMap;

use parser::{Parser, ParserError, expr::Expr};

mod parser;

fn main() {
	let equation: &str = "6 / 2 * (1 + 2) + ln(log_2_(2^e)) / pi";
	let mut test_parse: Parser<'_> = Parser::new(equation);
	let expr: Result<Expr, ParserError> = test_parse.parse_expr();
	println!("{equation}");
	println!("{expr:?}");
	println!(
		"{:?}",
		expr.expect("REASON")
			.execute(&HashMap::new(), &HashMap::new())
	);
}

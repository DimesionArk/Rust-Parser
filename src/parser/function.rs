use std::collections::HashMap;
use crate::parser::expr::Expr;

#[derive(Debug, Clone)]
pub struct Function {
        equation: Expr,
        param_variables: Vec<char>,
        all_variables: HashMap<char, Expr>
    }

impl Function {
    pub fn run(self, params: Vec<Expr>) -> f64 {
    	if params.len() != self.param_variables.len() {
        panic!("Incorrect number of parameters: {} insted of {}", params.len(), self.param_variables.len())
       	}

        let mut variables = HashMap::new();
    	for i in 0..params.len() {
            variables.insert(
            	self.param_variables[i],
        		params[i].clone()
        	);
    	}

		for name in self.all_variables.keys() {
			let value = self.all_variables.get(name).unwrap().clone();
        	variables.insert(
				*name,
				value
			);
    	}

	    self.equation.execute(&variables, &HashMap::new())
    }
}
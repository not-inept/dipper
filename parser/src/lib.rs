use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidExpression(String),
    VariableUnassigned(String),
    IsNotOperator(String)
}

#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Multiply,
    Divide,
    Exponent,
    Number,
    LParen,
    RParen,
    Variable(String),
    Value(f64)
}
impl Token {
    pub fn new(symbol : String) -> Token {
        let token = symbol.as_str();
        match token {
            "+" => return Token::Plus,
            "-" => return Token::Minus,
            "*" => return Token::Multiply,
            "/" => return Token::Divide,
            "^" => return Token::Exponent,
            "(" => return Token::LParen,
            ")" => return Token::RParen,
            _ => {
                match symbol.parse() {
                    Ok(n) => return Token::Value(n),
                    Err(_) => return Token::Variable(symbol.clone())
                }
            }
        }
    }
    pub fn assoc_right(&self) -> bool {
        match *self {
            Token::Exponent => true,
            _ => false     
        }
    }
    pub fn assoc_left(&self) -> bool {
        match *self {
            Token::Multiply |
            Token::Divide |
            Token::Plus |
            Token::Minus => true,
            _ => false     
        }
    }
    pub fn prec(&self) -> u64 {
        match *self {
            Token::Plus | Token::Minus => 2,
            Token::Multiply | Token::Divide  => 3,
            Token::Exponent => 4,
            _ => 0     
        }
    }
    pub fn is_op(&self) -> bool {
        match *self {
            Token::Plus | 
            Token::Minus | 
            Token::Multiply | 
            Token::Divide | 
            Token::Exponent => true,
            _ => false
        }
    }
    pub fn is_val(&self) -> bool {
        match *self {
            Token::Value(_) | 
            Token::Variable(_) => true,
            _ => false
        }
    }
    pub fn is_lparen(&self) -> bool {
        match *self {
            Token::LParen => true,
            _ => false
        }
    }
    pub fn is_rparen(&self) -> bool {
        match *self {
            Token::RParen => true,
            _ => false
        }
    }
    pub fn eval(&self, left : f64, right : f64) -> f64 {
        match *self {
            Token::Plus => return left + right,
            Token::Minus => return left - right,
            Token::Multiply => return left * right, 
            Token::Divide => return left / right,
            Token::Exponent => return left.powf(right),
            _ => return 0.0
        }
    }
}

pub fn lex(expression_raw : String) -> Result<Vec<Token>, Error> {
    let mut chars = expression_raw.chars().peekable();
    let mut tokens = Vec::new();

    let mut buff = String::from("");
    while let Some(c) = chars.next() {
        let c_string = c.to_string();
        let c_token = Token::new(c_string.clone());
        if c_token.is_op() || c_token.is_lparen() || c_token.is_rparen() {
            if buff != String::from("") {
                tokens.push(Token::new(buff));
                buff = String::from("");
            }
            tokens.push(c_token);
        } else {
            buff += &c_string;
        }
    }
    if buff != String::from("") {
        tokens.push(Token::new(buff));
        buff = String::from("");
    }
    Ok(tokens)
}

pub struct Parser {
    infix: Vec<Token>,
    postfix: Vec<Token>,
    vars: Vec<String>,
    vals: HashMap<String, f64>
}
impl Parser {
    pub fn new(expression : String) -> Result<Parser, Error> {
        match lex(expression.clone()) {
            Ok(i) => {
                let mut vars = Vec::new();
                let infix = i.clone();
                for token in infix {
                    match token {
                        Token::Variable(v) => {
                            vars.push(v);
                        },
                        _ => {}
                    }
                }
                let mut p = Parser {
                    infix: i.clone(),
                    postfix: Parser::build_postfix(i),
                    vars: vars,
                    vals: HashMap::new()
                };
                Ok(p)
            },
            Err(e) => Err(e)
        }
    }
    pub fn vars(&self) -> Vec<String> {
        return self.vars.clone();
    }
    pub fn bind(&mut self, var : String, val : f64) {
        self.vals.insert(var, val);
    }
    fn build_postfix(infix : Vec<Token>) -> Vec<Token> {
        let mut op_stack : Vec<Token> = Vec::new();
        let mut out_queue : Vec<Token> = Vec::new();
        for token in infix {
            if token.is_val() {
                out_queue.push(token);
            } else if token.is_op() {
                while op_stack.len() > 0
                        && (
                            (token.assoc_left() && token.prec() <= op_stack.last().unwrap().prec())
                         || (token.assoc_right() && token.prec() < op_stack.last().unwrap().prec())
                        ) {
                    let val = op_stack.pop().unwrap();
                    out_queue.push(val);   
                }
                op_stack.push(token);
            } else if token.is_lparen() {
                op_stack.push(token);
            } else if token.is_rparen() {
                println!("op_stack:\n{:?}", op_stack);
                while op_stack.len() > 0 && !op_stack.last().unwrap().is_lparen() {
                    let val = op_stack.pop().unwrap();
                    out_queue.push(val);   
                }
                op_stack.pop();
            }
        }
        op_stack.reverse();
        for token in op_stack {
            out_queue.push(token);
        }
        println!("\n\nOut queue:\n{:?}\n\n", out_queue);
        // verify variables are filled
        // evaluate expression
        return out_queue;
    }
    pub fn eval(&self) -> f64 {
        let mut val_queue = Vec::new();
        let postfix = self.postfix.clone();
        for token in postfix {
            if token.is_val() {
                let s = match token {
                    Token::Value(v) => v,
                    Token::Variable(s) => 
                        *self.vals.get(&s).unwrap(),
                    _ => 0.0
                };
                val_queue.push(s);
            } else if token.is_op() {
                let right = val_queue.pop().unwrap();
                let left = val_queue.pop().unwrap();
                let res = token.eval(left, right);
                val_queue.push(res);
            }
        }
        return val_queue[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_lexing() {
        println!("Testing!");
        let test_s = String::from("2^(2+2)-5");
        let result = lex(test_s.clone());
        match result {
            Ok(v) => println!("{:?}", v),
            Err(_) => println!("Erred")
        }
        let p = Parser::new(test_s).unwrap();
        println!("Result: {}", p.eval());
        assert_eq!(true, true);
    }
}


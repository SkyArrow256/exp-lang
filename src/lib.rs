mod ast;
mod interpreter;
mod parser;

pub fn run(code: String) {
    let tree = dbg!(parser::parse(&code));
    match tree {
        Ok(tree) => {
            dbg!(interpreter::eval(tree));
        }
        Err(err) => println!("{err}"),
    }
}

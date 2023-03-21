mod compiler;

use crate::compiler::generator;
use crate::compiler::lambda::{Statement, Term};
use crate::compiler::transformer;
use std::rc::Rc;

fn main() {
    let program = vec![Statement::Declaration {
        name: String::from("fix"),
        value: Rc::new(Term::Abs {
            param: String::from("f"),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Abs {
                    param: String::from("x"),
                    body: Rc::new(Term::App {
                        func: Rc::new(Term::Var {
                            name: String::from("f"),
                        }),
                        arg: Rc::new(Term::Abs {
                            param: String::from("y"),
                            body: Rc::new(Term::App {
                                func: Rc::new(Term::App {
                                    func: Rc::new(Term::Var {
                                        name: String::from("x"),
                                    }),
                                    arg: Rc::new(Term::Var {
                                        name: String::from("x"),
                                    }),
                                }),
                                arg: Rc::new(Term::Var {
                                    name: String::from("y"),
                                }),
                            }),
                        }),
                    }),
                }),
                arg: Rc::new(Term::Abs {
                    param: String::from("x"),
                    body: Rc::new(Term::App {
                        func: Rc::new(Term::Var {
                            name: String::from("f"),
                        }),
                        arg: Rc::new(Term::Abs {
                            param: String::from("y"),
                            body: Rc::new(Term::App {
                                func: Rc::new(Term::App {
                                    func: Rc::new(Term::Var {
                                        name: String::from("x"),
                                    }),
                                    arg: Rc::new(Term::Var {
                                        name: String::from("x"),
                                    }),
                                }),
                                arg: Rc::new(Term::Var {
                                    name: String::from("y"),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        }),
    }];
    let program = transformer::transform(&program);
    let code = generator::generate(&program);
    println!("{}", code);
}

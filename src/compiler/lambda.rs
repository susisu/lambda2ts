use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone)]
pub enum Term {
    Var {
        name: String,
    },
    App {
        func: Rc<Term>,
        arg: Rc<Term>,
    },
    Abs {
        param: String,
        body: Rc<Term>,
    },
    Let {
        name: String,
        value: Rc<Term>,
        body: Rc<Term>,
    },
}

impl Term {
    pub fn free_vars(&self) -> HashSet<String> {
        match self {
            Term::Var { name } => HashSet::from([name.clone()]),
            Term::App { func, arg } => {
                let func_vs = func.free_vars();
                let arg_vs = arg.free_vars();
                func_vs.union(&arg_vs).cloned().collect()
            }
            Term::Abs { param, body } => {
                let mut body_vs = body.free_vars();
                body_vs.remove(param);
                body_vs
            }
            Term::Let { name, value, body } => {
                let value_vs = value.free_vars();
                let mut body_vs = body.free_vars();
                body_vs.remove(name);
                value_vs.union(&body_vs).cloned().collect()
            }
        }
    }
}

pub enum Statement {
    Declaration { name: String, value: Box<Term> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_var_free_vars() {
        let term = Term::Var {
            name: String::from("x"),
        };
        let expected = HashSet::from([String::from("x")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn test_term_app_free_vars() {
        let term = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("y"),
            }),
        };
        let expected = HashSet::from([String::from("x"), String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn test_term_abs_free_vars() {
        let term = Term::Abs {
            param: String::from("x"),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn test_term_let_free_vars() {
        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Var {
                name: String::from("y"),
            }),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("y"), String::from("z")]);
        assert_eq!(term.free_vars(), expected);

        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("x"), String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }
}

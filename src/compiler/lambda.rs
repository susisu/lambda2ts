use std::collections::HashSet;

pub enum Term {
    Var {
        name: String,
    },
    App {
        func: Box<Term>,
        arg: Box<Term>,
    },
    Abs {
        param: String,
        body: Box<Term>,
    },
    Let {
        name: String,
        value: Box<Term>,
        body: Box<Term>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_var_free_vars() {
        let term = Term::Var {
            name: String::from("x"),
        };
        let expected = HashSet::from([String::from("x")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn term_app_free_vars() {
        let term = Term::App {
            func: Box::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Box::new(Term::Var {
                name: String::from("y"),
            }),
        };
        let expected = HashSet::from([String::from("x"), String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn term_abs_free_vars() {
        let term = Term::Abs {
            param: String::from("x"),
            body: Box::new(Term::App {
                func: Box::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Box::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn term_let_free_vars() {
        let term = Term::Let {
            name: String::from("x"),
            value: Box::new(Term::Var {
                name: String::from("y"),
            }),
            body: Box::new(Term::App {
                func: Box::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Box::new(Term::Var {
                    name: String::from("z"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("y"), String::from("z")]);
        assert_eq!(term.free_vars(), expected);

        let term = Term::Let {
            name: String::from("x"),
            value: Box::new(Term::Var {
                name: String::from("x"),
            }),
            body: Box::new(Term::App {
                func: Box::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Box::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = HashSet::from([String::from("x"), String::from("y")]);
        assert_eq!(term.free_vars(), expected);
    }
}

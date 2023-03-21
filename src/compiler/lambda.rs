use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
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
                let func_fvs = func.free_vars();
                let arg_fvs = arg.free_vars();
                func_fvs.union(&arg_fvs).cloned().collect()
            }
            Term::Abs { param, body } => {
                let mut body_fvs = body.free_vars();
                body_fvs.remove(param);
                body_fvs
            }
            Term::Let { name, value, body } => {
                let value_fvs = value.free_vars();
                let mut body_fvs = body.free_vars();
                body_fvs.remove(name);
                value_fvs.union(&body_fvs).cloned().collect()
            }
        }
    }

    pub fn subst(&self, name: &String, term: &Term) -> Term {
        match self {
            Term::Var { name: var_name } => {
                if var_name == name {
                    term.clone()
                } else {
                    self.clone()
                }
            }
            Term::App { func, arg } => Term::App {
                func: Rc::new(func.subst(name, term)),
                arg: Rc::new(arg.subst(name, term)),
            },
            Term::Abs { param, body } => Term::Abs {
                param: param.clone(),
                body: if param == name {
                    Rc::clone(body)
                } else {
                    Rc::new(body.subst(name, term))
                },
            },
            Term::Let {
                name: let_name,
                value,
                body,
            } => Term::Let {
                name: let_name.clone(),
                value: Rc::new(value.subst(name, term)),
                body: if let_name == name {
                    Rc::clone(body)
                } else {
                    Rc::new(body.subst(name, term))
                },
            },
        }
    }
}

#[cfg(test)]
mod tests_term_free_vars {
    use super::*;

    #[test]
    fn test_var() {
        let term = Term::Var {
            name: String::from("x"),
        };
        let expected = HashSet::from([String::from("x")]);
        assert_eq!(term.free_vars(), expected);
    }

    #[test]
    fn test_app() {
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
    fn test_abs() {
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
    fn test_let() {
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

#[cfg(test)]
mod tests_term_subst {
    use super::*;

    #[test]
    fn test_var() {
        let term = Term::Var {
            name: String::from("x"),
        };
        let subst_term = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("y"),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("z"),
            }),
        };
        assert_eq!(term.subst(&String::from("x"), &subst_term), subst_term);
        assert_eq!(term.subst(&String::from("y"), &subst_term), term);
    }

    #[test]
    fn test_app() {
        let term = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("y"),
            }),
        };
        let subst_term = Term::Var {
            name: String::from("z"),
        };

        let expected = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("z"),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("y"),
            }),
        };
        assert_eq!(term.subst(&String::from("x"), &subst_term), expected);

        let expected = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("z"),
            }),
        };
        assert_eq!(term.subst(&String::from("y"), &subst_term), expected);
    }

    #[test]
    fn test_abs() {
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
        let subst_term = Term::Var {
            name: String::from("z"),
        };

        assert_eq!(term.subst(&String::from("x"), &subst_term), term);

        let expected = Term::Abs {
            param: String::from("x"),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
            }),
        };
        assert_eq!(term.subst(&String::from("y"), &subst_term), expected);
    }

    #[test]
    fn test_let() {
        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
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
        let subst_term = Term::Var {
            name: String::from("z"),
        };

        let expected = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
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
        assert_eq!(term.subst(&String::from("x"), &subst_term), expected);

        let expected = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
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
        assert_eq!(term.subst(&String::from("y"), &subst_term), expected);
    }
}

pub enum Statement {
    Declaration { name: String, value: Box<Term> },
}

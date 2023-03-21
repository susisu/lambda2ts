use crate::compiler::lambda::Term;
use std::collections::HashSet;
use std::rc::Rc;

fn find_fresh_var(env: &HashSet<String>, prefix: &String) -> String {
    let mut var = prefix.clone();
    let mut i = 0;
    loop {
        if !env.contains(&var) {
            return var;
        }
        var = format!("{prefix}{i}");
        i += 1;
    }
}

#[cfg(test)]
mod tests_find_fresh_var {
    use super::*;

    #[test]
    fn test() {
        let env = HashSet::from([String::from("x"), String::from("y"), String::from("y0")]);
        assert_eq!(find_fresh_var(&env, &String::from("x")), String::from("x0"));
        assert_eq!(find_fresh_var(&env, &String::from("y")), String::from("y1"));
        assert_eq!(find_fresh_var(&env, &String::from("z")), String::from("z"));
    }
}

fn is_term_app(term: &Term) -> bool {
    match term {
        Term::Var { name: _ } => true,
        Term::App { func, arg } => is_term_app(func) && is_term_app(arg),
        Term::Abs { param: _, body: _ } => false,
        Term::Let {
            name: _,
            value: _,
            body: _,
        } => false,
    }
}

fn normalize_app(term: &Term) -> Term {
    match term {
        Term::Var { name: _ } => term.clone(),
        Term::App { func, arg } => {
            if !is_term_app(func) {
                match func.as_ref() {
                    Term::Var { name: _ } => normalize_app(&Term::App {
                        func: Rc::new(normalize_app(func)),
                        arg: Rc::clone(arg),
                    }),
                    Term::App { func: _, arg: _ } => normalize_app(&Term::App {
                        func: Rc::new(normalize_app(func)),
                        arg: Rc::clone(arg),
                    }),
                    Term::Abs { param, body } => {
                        let arg_fvs = arg.free_vars();
                        let new_name = find_fresh_var(&arg_fvs, &String::from("v"));
                        Term::Let {
                            name: new_name.clone(),
                            value: Rc::new(Term::Abs {
                                param: param.clone(),
                                body: Rc::new(normalize_app(body)),
                            }),
                            body: Rc::new(normalize_app(&Term::App {
                                func: Rc::new(Term::Var {
                                    name: new_name.clone(),
                                }),
                                arg: Rc::clone(arg),
                            })),
                        }
                    }
                    Term::Let { name, value, body } => {
                        let arg_fvs = arg.free_vars();
                        if arg_fvs.contains(name) {
                            let new_name = find_fresh_var(&arg_fvs, name);
                            Term::Let {
                                name: new_name.clone(),
                                value: Rc::new(normalize_app(value)),
                                body: Rc::new(normalize_app(&Term::App {
                                    func: Rc::new(body.subst(
                                        name,
                                        &Term::Var {
                                            name: new_name.clone(),
                                        },
                                    )),
                                    arg: Rc::clone(arg),
                                })),
                            }
                        } else {
                            Term::Let {
                                name: name.clone(),
                                value: Rc::new(normalize_app(value)),
                                body: Rc::new(normalize_app(&Term::App {
                                    func: Rc::clone(body),
                                    arg: Rc::clone(arg),
                                })),
                            }
                        }
                    }
                }
            } else if !is_term_app(arg) {
                match arg.as_ref() {
                    Term::Var { name: _ } => normalize_app(&Term::App {
                        func: Rc::clone(func),
                        arg: Rc::new(normalize_app(arg)),
                    }),
                    Term::App { func: _, arg: _ } => normalize_app(&Term::App {
                        func: Rc::clone(func),
                        arg: Rc::new(normalize_app(arg)),
                    }),
                    Term::Abs { param, body } => {
                        let func_fvs = func.free_vars();
                        let new_name = find_fresh_var(&func_fvs, &String::from("v"));
                        Term::Let {
                            name: new_name.clone(),
                            value: Rc::new(Term::Abs {
                                param: param.clone(),
                                body: Rc::new(normalize_app(body)),
                            }),
                            // body is already normalized
                            body: Rc::new(Term::App {
                                func: Rc::clone(func),
                                arg: Rc::new(Term::Var {
                                    name: new_name.clone(),
                                }),
                            }),
                        }
                    }
                    Term::Let { name, value, body } => {
                        let func_fvs = func.free_vars();
                        if func_fvs.contains(name) {
                            let new_name = find_fresh_var(&func_fvs, name);
                            Term::Let {
                                name: new_name.clone(),
                                value: Rc::new(normalize_app(value)),
                                body: Rc::new(normalize_app(&Term::App {
                                    func: Rc::clone(func),
                                    arg: Rc::new(body.subst(
                                        name,
                                        &Term::Var {
                                            name: new_name.clone(),
                                        },
                                    )),
                                })),
                            }
                        } else {
                            Term::Let {
                                name: name.clone(),
                                value: Rc::new(normalize_app(value)),
                                body: Rc::new(normalize_app(&Term::App {
                                    func: Rc::clone(func),
                                    arg: Rc::clone(body),
                                })),
                            }
                        }
                    }
                }
            } else {
                term.clone()
            }
        }
        Term::Abs { param, body } => Term::Abs {
            param: param.clone(),
            body: Rc::new(normalize_app(body)),
        },
        Term::Let { name, value, body } => Term::Let {
            name: name.clone(),
            value: Rc::new(normalize_app(value)),
            body: Rc::new(normalize_app(body)),
        },
    }
}

#[cfg(test)]
mod tests_normalize_app {
    use super::*;

    #[test]
    fn test_app_normal() {
        let term = Term::App {
            func: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("z"),
            }),
        };
        assert_eq!(normalize_app(&term), term);
    }

    #[test]
    fn test_app_abs() {
        let term = Term::App {
            func: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("x"),
            }),
        };
        let expected = Term::Let {
            name: String::from("v"),
            value: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
            }),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("v"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        assert_eq!(normalize_app(&term), expected);

        let term = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("v"),
            value: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
            }),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("v"),
                }),
            }),
        };
        assert_eq!(normalize_app(&term), expected);
    }

    #[test]
    fn test_app_let() {
        let term = Term::App {
            func: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
            arg: Rc::new(Term::Var {
                name: String::from("x"),
            }),
        };
        let expected = Term::Let {
            name: String::from("x0"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
            }),
            body: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("x0"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        assert_eq!(normalize_app(&term), expected);

        let term = Term::App {
            func: Rc::new(Term::Var {
                name: String::from("x"),
            }),
            arg: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("x0"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("y"),
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
                    name: String::from("x0"),
                }),
            }),
        };
        assert_eq!(normalize_app(&term), expected);
    }
}

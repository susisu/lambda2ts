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

fn is_term_abs(term: &Term) -> bool {
    match term {
        Term::Var { name: _ } => true,
        Term::App { func, arg } => is_term_abs(func) && is_term_abs(arg),
        Term::Abs { param: _, body } => is_term_abs(body),
        Term::Let {
            name: _,
            value: _,
            body: _,
        } => false,
    }
}

fn normalize_abs(term: &Term) -> Term {
    match term {
        Term::Var { name: _ } => term.clone(),
        // assuming term is already normalized by normalize_app
        Term::App { func: _, arg: _ } => term.clone(),
        Term::Abs { param, body } => {
            match body.as_ref() {
                Term::Var { name: _ } => term.clone(),
                // assuming body is normalized by normalize_app
                Term::App { func: _, arg: _ } => term.clone(),
                Term::Abs { param: _, body: _ } => {
                    if is_term_abs(body) {
                        term.clone()
                    } else {
                        normalize_abs(&Term::Abs {
                            param: param.clone(),
                            body: Rc::new(normalize_abs(body)),
                        })
                    }
                }
                Term::Let {
                    name: let_name,
                    value: let_value,
                    body: let_body,
                } => {
                    let let_value_fvs = let_value.free_vars();
                    let new_let_name = if let_name == param {
                        let let_body_fvs = let_body.free_vars();
                        find_fresh_var(&let_body_fvs, let_name)
                    } else {
                        let_name.clone()
                    };
                    if let_value_fvs.contains(param) {
                        Term::Let {
                            name: new_let_name.clone(),
                            value: Rc::new(normalize_abs(&Term::Abs {
                                param: param.clone(),
                                body: Rc::clone(let_value),
                            })),
                            body: Rc::new(normalize_abs(&Term::Abs {
                                param: param.clone(),
                                body: Rc::new(let_body.subst(
                                    let_name,
                                    &Term::App {
                                        func: Rc::new(Term::Var {
                                            name: new_let_name.clone(),
                                        }),
                                        arg: Rc::new(Term::Var {
                                            name: param.clone(),
                                        }),
                                    },
                                )),
                            })),
                        }
                    } else {
                        Term::Let {
                            name: new_let_name.clone(),
                            value: Rc::new(normalize_abs(let_value)),
                            body: Rc::new(normalize_abs(&&Term::Abs {
                                param: param.clone(),
                                body: Rc::new(let_body.subst(
                                    let_name,
                                    &Term::Var {
                                        name: new_let_name.clone(),
                                    },
                                )),
                            })),
                        }
                    }
                }
            }
        }
        Term::Let { name, value, body } => Term::Let {
            name: name.clone(),
            value: Rc::new(normalize_abs(value)),
            body: Rc::new(normalize_abs(body)),
        },
    }
}

#[cfg(test)]
mod tests_normalize_abs {
    use super::*;

    #[test]
    fn test_abs_normal() {
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
        assert_eq!(normalize_abs(&term), term);
    }

    #[test]
    fn test_abs_let() {
        let term = Term::Abs {
            param: String::from("x"),
            body: Rc::new(Term::Let {
                name: String::from("y"),
                value: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("y"),
            value: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
            body: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("x"),
                    }),
                }),
            }),
        };
        assert_eq!(normalize_abs(&term), expected);

        let term = Term::Abs {
            param: String::from("x"),
            body: Rc::new(Term::Let {
                name: String::from("y"),
                value: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("y"),
            value: Rc::new(Term::Var {
                name: String::from("z"),
            }),
            body: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
        };
        assert_eq!(normalize_abs(&term), expected);

        let term = Term::Abs {
            param: String::from("x"),
            body: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("x0"),
            value: Rc::new(Term::Var {
                name: String::from("y"),
            }),
            body: Rc::new(Term::Abs {
                param: String::from("x"),
                body: Rc::new(Term::Var {
                    name: String::from("x0"),
                }),
            }),
        };
        assert_eq!(normalize_abs(&term), expected);
    }
}

fn is_term_let(term: &Term) -> bool {
    match term {
        Term::Var { name: _ } => true,
        Term::App { func, arg } => is_term_let(func) && is_term_let(arg),
        Term::Abs { param: _, body } => is_term_let(body),
        Term::Let {
            name: _,
            value: _,
            body: _,
        } => false,
    }
}

fn normalize_let(term: &Term) -> Term {
    match term {
        Term::Var { name: _ } => term.clone(),
        // assuming term is already normalized by normalize_app
        Term::App { func: _, arg: _ } => term.clone(),
        // assuming term is already normalized by normalize_abs
        Term::Abs { param: _, body: _ } => term.clone(),
        Term::Let { name, value, body } => {
            match value.as_ref() {
                Term::Var { name: _ } => Term::Let {
                    name: name.clone(),
                    value: Rc::clone(value),
                    body: Rc::new(normalize_let(body)),
                },
                // assuming value is already normalized by normalize_app
                Term::App { func: _, arg: _ } => Term::Let {
                    name: name.clone(),
                    value: Rc::clone(value),
                    body: Rc::new(normalize_let(body)),
                },
                // assuming value is already normalized by normalize_abs
                Term::Abs { param: _, body: _ } => Term::Let {
                    name: name.clone(),
                    value: Rc::clone(value),
                    body: Rc::new(normalize_let(body)),
                },
                Term::Let {
                    name: inner_name,
                    value: inner_value,
                    body: inner_body,
                } => {
                    if is_term_let(&inner_value) {
                        let new_name = if inner_name == name {
                            let inner_body_fvs = inner_body.free_vars();
                            find_fresh_var(&inner_body_fvs, inner_name)
                        } else {
                            inner_name.clone()
                        };
                        Term::Let {
                            name: new_name.clone(),
                            value: Rc::clone(inner_value),
                            body: Rc::new(normalize_let(&Term::Let {
                                name: name.clone(),
                                value: Rc::new(
                                    inner_body.subst(inner_name, &Term::Var { name: new_name }),
                                ),
                                body: Rc::clone(body),
                            })),
                        }
                    } else {
                        normalize_let(&Term::Let {
                            name: name.clone(),
                            value: Rc::new(normalize_let(value)),
                            body: Rc::clone(body),
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_normalize_let {
    use super::*;

    #[test]
    fn test_let_normal() {
        let term = Term::Let {
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
        };
        assert_eq!(normalize_let(&term), term);
    }

    #[test]
    fn test_let_let() {
        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Let {
                name: String::from("y"),
                value: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("w"),
                    }),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
            }),
            body: Rc::new(Term::Var {
                name: String::from("x"),
            }),
        };
        let expected = Term::Let {
            name: String::from("y"),
            value: Rc::new(Term::App {
                func: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
                arg: Rc::new(Term::Var {
                    name: String::from("w"),
                }),
            }),
            body: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::Var {
                    name: String::from("y"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        assert_eq!(normalize_let(&term), expected);

        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Let {
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
            body: Rc::new(Term::Var {
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
            body: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::Var {
                    name: String::from("x0"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        assert_eq!(normalize_let(&term), expected);
    }
}

fn mangle(term: &Term, env: &HashSet<String>) -> Term {
    match term {
        Term::Var { name: _ } => term.clone(),
        Term::App { func: _, arg: _ } => term.clone(),
        Term::Abs { param: _, body: _ } => term.clone(),
        Term::Let { name, value, body } => {
            let new_name = if env.contains(name) {
                find_fresh_var(env, name)
            } else {
                name.clone()
            };
            let mut new_env = env.clone();
            new_env.insert(name.clone());
            Term::Let {
                name: new_name.clone(),
                value: Rc::clone(value),
                body: Rc::new(mangle(
                    &body.subst(
                        name,
                        &Term::Var {
                            name: new_name.clone(),
                        },
                    ),
                    &new_env,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests_mangle {
    use super::*;

    #[test]
    fn test_mangle() {
        let term = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Var {
                name: String::from("y"),
            }),
            body: Rc::new(Term::Let {
                name: String::from("x"),
                value: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            }),
        };
        let expected = Term::Let {
            name: String::from("x"),
            value: Rc::new(Term::Var {
                name: String::from("y"),
            }),
            body: Rc::new(Term::Let {
                name: String::from("x0"),
                value: Rc::new(Term::Var {
                    name: String::from("z"),
                }),
                body: Rc::new(Term::Var {
                    name: String::from("x0"),
                }),
            }),
        };
        assert_eq!(mangle(&term, &HashSet::new()), expected);
    }
}

pub fn transform(term: &Term) -> Term {
    let term = normalize_app(term);
    let term = normalize_abs(&term);
    let term = normalize_let(&term);
    mangle(&term, &HashSet::new())
}

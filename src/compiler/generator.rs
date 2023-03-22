use std::{rc::Rc, vec::Vec};

use super::lambda::{Statement, Term};

fn generate_term(term: &Term) -> String {
    match term {
        Term::Var { name } => name.clone(),
        Term::App { func, arg } => {
            let func = generate_term(func);
            let arg = generate_term(arg);
            format!("App<{func}, {arg}>")
        }
        Term::Abs { param: _, body: _ } => panic!("undefined"),
        Term::Let {
            name: _,
            value: _,
            body: _,
        } => panic!("undefined"),
    }
}

fn generate_statement(stmt: &Statement) -> String {
    match stmt {
        Statement::Declaration { name, value } => match value.as_ref() {
            Term::Var { name: _ } => {
                let ret = generate_term(value);
                format!("type {name} = {ret};\n")
            }
            Term::App { func: _, arg: _ } => {
                let ret = generate_term(value);
                format!("type {name} = {ret};\n")
            }
            Term::Abs { param: _, body: _ } => {
                let mut res = String::new();
                let mut current_term = value.as_ref().clone();
                let mut if_args = String::new();
                let mut depth = 0;

                while let Term::Abs { param, body } = current_term {
                    let if_sig = if depth == 0 {
                        name.clone()
                    } else {
                        format!("{name}${depth}<{if_args}>")
                    };

                    let next_depth = depth + 1;

                    let if_ret = if depth == 0 {
                        format!("{name}${next_depth}<this[\"arg\"]>")
                    } else {
                        format!("{name}${next_depth}<{if_args}, this[\"arg\"]>")
                    };

                    let if_code = format!("interface {if_sig} extends Fun {{ ret: {if_ret} }}\n");
                    res.push_str(if_code.as_str());

                    current_term = body.as_ref().clone();
                    if depth == 0 {
                        if_args.push_str(param.as_str());
                    } else {
                        if_args.push_str(format!(", {param}").as_str());
                    }
                    depth = next_depth;
                }

                let type_name = format!("{name}${depth}");
                let type_ret = generate_term(&current_term);
                let type_code = format!("type {type_name}<{if_args}> = {type_ret};\n");
                res.push_str(type_code.as_str());

                res
            }
            Term::Let {
                name: _,
                value: _,
                body: _,
            } => {
                let mut res = String::new();
                let mut current_term = value.as_ref().clone();

                while let Term::Let {
                    name: inner_name,
                    value: inner_value,
                    body: inner_body,
                } = current_term
                {
                    let new_name = String::from(format!("{name}${inner_name}"));
                    let hyp_stmt = Statement::Declaration {
                        name: new_name.clone(),
                        value: Rc::clone(&inner_value),
                    };
                    let code = &generate_statement(&hyp_stmt);
                    res.push_str(code);

                    current_term = inner_body.subst(
                        &inner_name,
                        &Term::Var {
                            name: new_name.clone(),
                        },
                    );
                }

                let hyp_stmt = Statement::Declaration {
                    name: name.clone(),
                    value: Rc::new(current_term.clone()),
                };
                let code = &generate_statement(&hyp_stmt);
                res.push_str(code);

                res
            }
        },
    }
}

pub fn generate(program: &Vec<Statement>) -> String {
    let mut res = String::new();
    res.push_str("interface Fun { arg: unknown; ret: unknown }\n");
    res.push_str("type App<F, X> = F extends Fun ? (F & { arg: X })[\"ret\"] : never;\n");
    for stmt in program.iter() {
        res.push_str(generate_statement(stmt).as_str());
    }
    res
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_generate() {
        let program = vec![
            Statement::Declaration {
                name: String::from("foo"),
                value: Rc::new(Term::Var {
                    name: String::from("x"),
                }),
            },
            Statement::Declaration {
                name: String::from("bar"),
                value: Rc::new(Term::App {
                    func: Rc::new(Term::Var {
                        name: String::from("x"),
                    }),
                    arg: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                }),
            },
            Statement::Declaration {
                name: String::from("baz"),
                value: Rc::new(Term::Abs {
                    param: String::from("x"),
                    body: Rc::new(Term::Abs {
                        param: String::from("y"),
                        body: Rc::new(Term::App {
                            func: Rc::new(Term::Var {
                                name: String::from("x"),
                            }),
                            arg: Rc::new(Term::Var {
                                name: String::from("y"),
                            }),
                        }),
                    }),
                }),
            },
            Statement::Declaration {
                name: String::from("qux"),
                value: Rc::new(Term::Let {
                    name: String::from("x"),
                    value: Rc::new(Term::Var {
                        name: String::from("y"),
                    }),
                    body: Rc::new(Term::Let {
                        name: String::from("z"),
                        value: Rc::new(Term::Var {
                            name: String::from("x"),
                        }),
                        body: Rc::new(Term::App {
                            func: Rc::new(Term::Var {
                                name: String::from("x"),
                            }),
                            arg: Rc::new(Term::Var {
                                name: String::from("z"),
                            }),
                        }),
                    }),
                }),
            },
        ];
        let expected = indoc! {"
            interface Fun { arg: unknown; ret: unknown }
            type App<F, X> = F extends Fun ? (F & { arg: X })[\"ret\"] : never;
            type foo = x;
            type bar = App<x, y>;
            interface baz extends Fun { ret: baz$1<this[\"arg\"]> }
            interface baz$1<x> extends Fun { ret: baz$2<x, this[\"arg\"]> }
            type baz$2<x, y> = App<x, y>;
            type qux$x = y;
            type qux$z = qux$x;
            type qux = App<qux$x, qux$z>;
        "};
        assert_eq!(generate(&program), expected);
    }
}

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, multispace1},
    combinator::{eof, map_opt, recognize, success, value},
    error::{Error, ParseError},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, tuple},
    Finish, IResult,
};
use once_cell::sync::Lazy;
use std::{collections::HashSet, rc::Rc};

use super::lambda::{Statement, Term};

fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((tag("(*"), take_until("*)"), tag("*)"))))(input)
}

fn ws<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), many0_count(alt((value((), multispace1), comment))))(input)
}

fn token<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(ws, inner, ws)
}

static RESERVED_WORDS: Lazy<HashSet<&str>> = Lazy::new(|| HashSet::from(["fun", "let", "in"]));

fn is_reserved_word(input: &str) -> bool {
    RESERVED_WORDS.contains(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    map_opt(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        |id| {
            if !is_reserved_word(id) {
                Some(id)
            } else {
                None
            }
        },
    )(input)
}

fn parens<'a, F: 'a, O, E: ParseError<&'a str> + 'a>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(token(tag("(")), inner, token(tag(")")))
}

fn term(input: &str) -> IResult<&str, Term> {
    alt((term_app, term_abs, term_let, parens(term)))(input)
}

fn aterm(input: &str) -> IResult<&str, Term> {
    alt((term_var, parens(term)))(input)
}

fn term_var(input: &str) -> IResult<&str, Term> {
    let (input, name) = token(identifier)(input)?;
    let result = Term::Var {
        name: String::from(name),
    };
    success(result)(input)
}

fn term_app(input: &str) -> IResult<&str, Term> {
    let (input, func) = aterm(input)?;
    let (input, args) = many0(aterm)(input)?;
    let result = args.into_iter().fold(func, |acc, arg| Term::App {
        func: Rc::new(acc),
        arg: Rc::new(arg),
    });
    success(result)(input)
}

fn term_abs(input: &str) -> IResult<&str, Term> {
    let (input, _) = token(tag("fun"))(input)?;
    let (input, params) = many1(token(identifier))(input)?;
    let (input, _) = token(tag("->"))(input)?;
    let (input, body) = term(input)?;
    let result = params.iter().rev().fold(body, |acc, &param| Term::Abs {
        param: String::from(param),
        body: Rc::new(acc),
    });
    success(result)(input)
}

fn term_let(input: &str) -> IResult<&str, Term> {
    let (input, _) = token(tag("let"))(input)?;
    let (input, name) = token(identifier)(input)?;
    let (input, params) = many0(token(identifier))(input)?;
    let (input, _) = token(tag("="))(input)?;
    let (input, value) = term(input)?;
    let (input, _) = token(tag("in"))(input)?;
    let (input, body) = term(input)?;
    let result = Term::Let {
        name: String::from(name),
        value: Rc::new(params.iter().rev().fold(value, |acc, &param| Term::Abs {
            param: String::from(param),
            body: Rc::new(acc),
        })),
        body: Rc::new(body),
    };
    success(result)(input)
}

fn stmt(input: &str) -> IResult<&str, Statement> {
    stmt_decl(input)
}

fn stmt_decl(input: &str) -> IResult<&str, Statement> {
    let (input, _) = token(tag("let"))(input)?;
    let (input, name) = token(identifier)(input)?;
    let (input, params) = many0(token(identifier))(input)?;
    let (input, _) = token(tag("="))(input)?;
    let (input, value) = term(input)?;
    let result = Statement::Declaration {
        name: String::from(name),
        value: Rc::new(params.iter().rev().fold(value, |acc, &param| Term::Abs {
            param: String::from(param),
            body: Rc::new(acc),
        })),
    };
    success(result)(input)
}

fn program(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, stmts) = separated_list0(many1(token(tag(";"))), stmt)(input)?;
    let (input, _) = many0(token(tag(";")))(input)?;
    let (input, _) = eof(input)?;
    success(stmts)(input)
}

fn parse(input: &str) -> Result<Vec<Statement>, Error<&str>> {
    program(input).finish().map(|(_, output)| output)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_parse() {
        let input = indoc! {"
          let foo = x;
          let bar = x (y z);
          let baz = fun x y -> x y;
          let qux = let f x = y in z;
        "};
        let output = parse(input).unwrap();
        let expected = vec![
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
                    arg: Rc::new(Term::App {
                        func: Rc::new(Term::Var {
                            name: String::from("y"),
                        }),
                        arg: Rc::new(Term::Var {
                            name: String::from("z"),
                        }),
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
                    name: String::from("f"),
                    value: Rc::new(Term::Abs {
                        param: String::from("x"),
                        body: Rc::new(Term::Var {
                            name: String::from("y"),
                        }),
                    }),
                    body: Rc::new(Term::Var {
                        name: String::from("z"),
                    }),
                }),
            },
        ];
        assert_eq!(output, expected);
    }
}

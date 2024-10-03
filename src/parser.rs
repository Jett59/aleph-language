use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, multispace0},
    combinator::map_res,
    error::ParseError,
    multi::{fold_many1, many0, separated_list0},
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Variable(String),

    Negate(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Power(Box<Expression>, Box<Expression>),

    ApplyFunction {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum TopLevelStatement {
    FunctionTypeDeclaration {
        name: String,
        domain: Type,
        codomain: Type,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Expression,
    },
}

fn with_whitespace<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(
    f: F,
) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

fn parse_name(input: &str) -> IResult<&str, &str> {
    with_whitespace(alpha1).parse(input)
}

fn parse_typ(input: &str) -> IResult<&str, Type> {
    parse_name(input).map(|(input, name)| (input, Type::Named(name.to_string())))
}

fn parse_function_type_declaration(input: &str) -> IResult<&str, TopLevelStatement> {
    let (input, name) = parse_name(input)?;
    let (input, _) = with_whitespace(tag(":")).parse(input)?;
    let (input, domain) = parse_typ(input)?;
    let (input, _) = with_whitespace(tag("->")).parse(input)?;
    let (input, codomain) = parse_typ(input)?;
    Ok((
        input,
        TopLevelStatement::FunctionTypeDeclaration {
            name: name.to_string(),
            domain,
            codomain,
        },
    ))
}

fn left_associative_operator_parser<'a, Error: ParseError<&'a str>>(
    operator: &str,
    parameter_parser: &mut impl Parser<&'a str, Expression, Error>,
    operator_constructor: impl Fn(Expression, Expression) -> Expression,
    input: &'a str,
) -> IResult<&'a str, Expression, Error> {
    let mut op_parser = with_whitespace(tag(operator));
    let (input, first) = parameter_parser.parse(input)?;
    fold_many1(
        move |input| {
            let (input, _) = op_parser.parse(input)?;
            parameter_parser.parse(input)
        },
        move || first.clone(),
        |lhs, rhs| operator_constructor(lhs, rhs),
    )
    .parse(input)
}

pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    let integer_parser = with_whitespace(map_res(digit1, |s: &str| {
        s.parse::<i64>().map(Expression::Integer)
    }));
    let variable_parser = with_whitespace(parse_name).map(|s| Expression::Variable(s.to_string()));
    let bracketed_expression = with_whitespace(delimited(tag("("), parse_expression, tag(")")));

    let negative_expression = with_whitespace(char('-'))
        .and(parse_expression)
        .map(|(_, expression)| Expression::Negate(Box::new(expression)));

    let mut atomic_expression = integer_parser.or(variable_parser).or(bracketed_expression).or(negative_expression);

    let mut possibly_apply_parser = move |input| {
        let (input, first) = atomic_expression.parse(input)?;
        with_whitespace(char('('))
            .and(separated_list0(
                with_whitespace(char(',')),
                parse_expression,
            ))
            .and(with_whitespace(char(')')))
            .map(|((_, arguments), _)| Expression::ApplyFunction {
                function: Box::new(first.clone()),
                arguments,
            })
            .parse(input)
            .or_else(|_| Ok((input, first)))
    };

    // TODO: switch to a right-associative parser
    let mut possibly_power_parser = move |input| {
        left_associative_operator_parser(
            "^",
            &mut possibly_apply_parser,
            |lhs, rhs| Expression::Power(Box::new(lhs), Box::new(rhs)),
            input,
        )
        .or_else(|_| possibly_apply_parser.parse(input))
    };

    let mut possibly_divide_parser = move |input| {
        left_associative_operator_parser(
            "/",
            &mut possibly_power_parser,
            |lhs, rhs| Expression::Divide(Box::new(lhs), Box::new(rhs)),
            input,
        )
        .or_else(|_| possibly_power_parser(input))
    };

    let mut possibly_multiply_parser = move |input| {
        left_associative_operator_parser(
            "*",
            &mut possibly_divide_parser,
            |lhs, rhs| Expression::Multiply(Box::new(lhs), Box::new(rhs)),
            input,
        )
        .or_else(|_| possibly_divide_parser(input))
    };

    let mut possibly_subtract_parser = move |input| {
        left_associative_operator_parser(
            "-",
            &mut possibly_multiply_parser,
            |lhs, rhs| Expression::Subtract(Box::new(lhs), Box::new(rhs)),
            input,
        )
        .or_else(|_| possibly_multiply_parser(input))
    };

    let mut possibly_add_parser = move |input| {
        left_associative_operator_parser(
            "+",
            &mut possibly_subtract_parser,
            |lhs, rhs| Expression::Add(Box::new(lhs), Box::new(rhs)),
            input,
        )
        .or_else(|_| possibly_subtract_parser(input))
    };

    possibly_add_parser.parse(input)
}

pub fn parse_function_definition(input: &str) -> IResult<&str, TopLevelStatement> {
    let (input, name) = parse_name(input)?;
    let (input, _) = with_whitespace(tag("(")).parse(input)?;
    let (input, parameters) = separated_list0(
        with_whitespace(tag(",")),
        parse_name.map(|name| name.to_string()),
    )(input)?;
    let (input, _) = with_whitespace(tag(")")).parse(input)?;
    let (input, _) = with_whitespace(tag("=")).parse(input)?;
    let (input, body) = parse_expression(input)?;
    Ok((
        input,
        TopLevelStatement::FunctionDefinition {
            name: name.to_string(),
            parameters,
            body,
        },
    ))
}

pub fn parse_top_level(input: &str) -> IResult<&str, Vec<TopLevelStatement>> {
    many0(alt((
        parse_function_type_declaration,
        parse_function_definition,
    )))
    .parse(input)
}

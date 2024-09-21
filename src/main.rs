use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, multispace0},
    error::ParseError,
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, Clone)]
enum Type {
    Named(String),
}

#[derive(Debug, Clone)]
enum TopLevelStatement {
    FunctionTypeDeclaration {
        name: String,
        domain: Type,
        codomain: Type,
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

fn parse_top_level(input: &str) -> IResult<&str, Vec<TopLevelStatement>> {
    many0(parse_function_type_declaration).parse(input)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_name = &args[1];
    let input = std::fs::read_to_string(file_name).unwrap();
    let (_, top_level) = parse_top_level(&input).unwrap();
    println!("{:?}", top_level);
}

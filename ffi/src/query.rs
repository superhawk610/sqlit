use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::{
        complete::{multispace0, multispace1},
        is_alphanumeric,
    },
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug)]
pub enum Query {
    Select {
        table: String,
        fields: SelectFields,
        distinct: bool,
    },
    // there's probably a better way to do this
    Invalid(String),
}

#[derive(Debug)]
pub enum SelectFields {
    All,
    Some(Vec<String>),
}

fn keyword(keyword: &'static str) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input: &str| map(tag_no_case(keyword), |s: &str| s.to_ascii_lowercase())(input)
}

fn is_valid_ident(c: char) -> bool {
    is_alphanumeric(c as u8) || c == '_'
}

fn ident(i: &str) -> IResult<&str, &str> {
    take_while1(is_valid_ident)(i)
}

fn quoted_ident(i: &str) -> IResult<&str, &str> {
    delimited(opt(tag("\"")), ident, opt(tag("\"")))(i)
}

fn parse_select(i: &str) -> IResult<&str, Query> {
    let (i, _) = keyword("select")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, k_distinct) = opt(terminated(
        alt((keyword("all"), keyword("distinct"))),
        multispace1,
    ))(i)?;
    let (i, fields) = alt((
        map(tag("*"), |_| SelectFields::All),
        map(
            separated_list1(delimited(multispace0, tag(","), multispace0), quoted_ident),
            |fields| SelectFields::Some(fields.iter().map(|s| s.to_string()).collect()),
        ),
    ))(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = keyword("from")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, table) = quoted_ident(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = opt(tag(";"))(i)?;

    let table = table.to_string();
    let distinct = k_distinct.as_deref() == Some("distinct");

    Ok((
        i,
        Query::Select {
            table,
            fields,
            distinct,
        },
    ))
}

impl Query {
    pub fn parse(input: &str) -> Self {
        let input = input.trim();
        match alt((parse_select,))(input) {
            Ok(("", query)) => query,
            Ok(_) => Self::Invalid("trailing input".into()),
            Err(err) => Self::Invalid(format!("{:?}", err)),
        }
    }
}

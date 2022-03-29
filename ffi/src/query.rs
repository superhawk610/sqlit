use crate::database::{Column, ColumnType, Table, Value};
use crate::error::SQLitError;
use nom::bytes::complete::is_not;
use nom::character::complete::digit1;
use nom::combinator::{all_consuming, map_res, recognize};
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::sequence::tuple;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, tag_no_case, take_while1},
    character::{
        complete::{alpha1, multispace0, multispace1},
        is_alphanumeric,
    },
    combinator::{map, opt, peek},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated},
    IResult as NomResult,
};

type IResult<I, O> = NomResult<I, O, QueryError<I>>;

#[derive(Debug, PartialEq)]
pub enum QueryError<I> {
    Validation(String),
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for QueryError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for QueryError<I> {
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum Query {
    Create {
        table: Table,
        if_not_exists: bool,
    },
    Select {
        table: String,
        fields: SelectFields,
        distinct: bool,
    },
}

#[derive(Debug, PartialEq)]
pub enum SelectFields {
    All,
    Some(Vec<String>),
}

macro_rules! invalid {
    ($msg: literal) => {
        Err(nom::Err::Error(QueryError::Validation($msg.to_string())))
    };
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

// TODO: this needs more robust numeric/string literal support
// see: https://github.com/Geal/nom/blob/main/examples/string.rs
// see: https://docs.rs/nom/latest/nom/recipes/index.html#floating-point-numbers
fn value(i: &str) -> IResult<&str, Value> {
    alt((
        map(delimited(tag("'"), is_not("'"), tag("'")), |s: &str| {
            Value::Text(s.to_string())
        }),
        map_res(
            recognize(tuple((digit1, tag("."), opt(digit1)))),
            |s: &str| s.parse().map(Value::Real),
        ),
        map_res(recognize(digit1), |s: &str| s.parse().map(Value::Integer)),
    ))(i)
}

fn column_type(i: &str) -> IResult<&str, ColumnType> {
    let (i, k_type) = alt((
        keyword("integer"),
        keyword("real"),
        keyword("text"),
        keyword("blob"),
    ))(i)?;

    match k_type.as_ref() {
        "integer" => Ok((i, ColumnType::Integer)),
        "real" => Ok((i, ColumnType::Real)),
        "text" => Ok((i, ColumnType::Text)),
        "blob" => Ok((i, ColumnType::Blob)),
        _ => unreachable!(),
    }
}

fn column(i: &str) -> IResult<&str, (Column, bool, bool)> {
    let mut primary_key = false;
    let mut autoincrement = false;

    let (i, name) = ident(i)?;
    let (i, _) = multispace1(i)?;
    let (i, t) = column_type(i)?;
    let (i, _) = multispace0(i)?;

    let mut column = Column {
        name: name.to_string(),
        t,
        default: None,
        allow_null: true,
        unique: false,
    };

    let (i, _) = separated_list0(
        multispace1,
        permutation((
            // PRIMARY KEY ( AUTOINCREMENT )?
            opt(map(
                tuple((
                    keyword("primary"),
                    multispace1,
                    keyword("key"),
                    opt(preceded(multispace1, keyword("autoincrement"))),
                )),
                |(_, _, _, k_autoincrement)| {
                    primary_key = true;
                    autoincrement = k_autoincrement.is_some();
                },
            )),
            // DEFAULT <value>
            opt(map(
                tuple((keyword("default"), multispace1, value)),
                |(_, _, v)| column.default = Some(v),
            )),
            // NOT NULL
            opt(map(
                tuple((keyword("not"), multispace1, keyword("null"))),
                |_| column.allow_null = false,
            )),
            // UNIQUE
            opt(map(keyword("unique"), |_| {
                column.unique = true;
            })),
        )),
    )(i)?;

    Ok((i, (column, primary_key, autoincrement)))
}

fn parse_create_columns(i: &str) -> IResult<&str, (Vec<Column>, usize, bool)> {
    let mut primary_key = None;
    let mut autoincrement = false;

    let (i, column_defs) =
        separated_list1(delimited(multispace0, tag(","), multispace0), column)(i)?;

    let mut columns = Vec::with_capacity(column_defs.len());
    for (idx, (column, is_primary_key, should_autoincrement)) in column_defs.into_iter().enumerate()
    {
        if is_primary_key {
            if primary_key.is_some() {
                return invalid!("only one primary key may be specified");
            }

            primary_key = Some(idx);
            autoincrement = should_autoincrement;
        }

        columns.push(column);
    }

    match primary_key {
        Some(primary_key) => Ok((i, (columns, primary_key, autoincrement))),
        None => invalid!("no primary key specified"),
    }
}

fn parse_create(i: &str) -> IResult<&str, Query> {
    let (i, _) = keyword("create")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = keyword("table")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, k_if_not_exists) = opt(terminated(
        tuple((
            keyword("if"),
            multispace1,
            keyword("not"),
            multispace1,
            keyword("exists"),
        )),
        multispace1,
    ))(i)?;
    let (i, name) = quoted_ident(i)?;
    let (i, _) = multispace1(i)?;
    let (i, (columns, primary_key, autoincrement)) = delimited(
        terminated(tag("("), multispace0),
        parse_create_columns,
        preceded(multispace0, tag(")")),
    )(i)?;
    let (i, _) = opt(preceded(multispace0, tag(";")))(i)?;

    let table = Table::new(name.to_string(), columns, primary_key, autoincrement);
    let if_not_exists = k_if_not_exists.is_some();

    Ok((
        i,
        Query::Create {
            table,
            if_not_exists,
        },
    ))
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
    let (i, _) = opt(preceded(multispace0, tag(";")))(i)?;

    let table = table.to_string();
    let distinct = k_distinct.as_deref() == Some("distinct");

    if fields == SelectFields::All && distinct {
        return invalid!("cannot select distinct on *");
    }

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
    pub fn parse(input: &str) -> Result<Self, SQLitError> {
        let input = input.trim();
        let (_, command) = map(peek(alpha1), |s: &str| s.to_ascii_lowercase())(input).map_err(
            |_: nom::Err<nom::error::Error<&str>>| SQLitError::InvalidQuery("empty input".into()),
        )?;

        let parser = match command.as_ref() {
            "select" => parse_select,
            "create" => parse_create,
            _ => return Err(SQLitError::InvalidQuery("unrecognized input".into())),
        };

        match all_consuming(parser)(input) {
            Ok((_, query)) => Ok(query),
            Err(err) => Err(SQLitError::InvalidQuery(format!("{:?}", err))),
        }
    }
}

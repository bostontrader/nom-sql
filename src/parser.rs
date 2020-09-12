use std::fmt;
use std::str;

use compound_select::{compound_selection, CompoundSelectStatement};
use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;
use select::{selection, SelectStatement};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SqlQuery {
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
}

impl fmt::Display for SqlQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlQuery::Select(ref select) => write!(f, "{}", select),
            _ => unimplemented!(),
        }
    }
}

pub fn sql_query(i: &[u8]) -> IResult<&[u8], SqlQuery> {
    alt((
        map(compound_selection, |cs| SqlQuery::CompoundSelect(cs)),
        map(selection, |s| SqlQuery::Select(s)),
    ))(i)
}

pub fn parse_query_bytes<T>(input: T) -> Result<SqlQuery, &'static str>
where
    T: AsRef<[u8]>,
{
    match sql_query(input.as_ref()) {
        Ok((_, o)) => Ok(o),
        Err(_) => Err("failed to parse query"),
    }
}

pub fn parse_query<T>(input: T) -> Result<SqlQuery, &'static str>
where
    T: AsRef<str>,
{
    parse_query_bytes(input.as_ref().trim().as_bytes())
}

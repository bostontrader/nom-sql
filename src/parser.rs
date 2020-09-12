use std::fmt;
use std::str;

use compound_select::{compound_selection, CompoundSelectStatement};
use delete::{deletion, DeleteStatement};
use insert::{insertion, InsertStatement};
use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;
use select::{selection, SelectStatement};
use update::{updating, UpdateStatement};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SqlQuery {
    Insert(InsertStatement),
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
}

impl fmt::Display for SqlQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlQuery::Select(ref select) => write!(f, "{}", select),
            SqlQuery::Insert(ref insert) => write!(f, "{}", insert),
            SqlQuery::Delete(ref delete) => write!(f, "{}", delete),
            SqlQuery::Update(ref update) => write!(f, "{}", update),
            _ => unimplemented!(),
        }
    }
}

pub fn sql_query(i: &[u8]) -> IResult<&[u8], SqlQuery> {
    alt((
        //map(creation, |c| SqlQuery::CreateTable(c)),
        map(insertion, |i| SqlQuery::Insert(i)),
        map(compound_selection, |cs| SqlQuery::CompoundSelect(cs)),
        map(selection, |s| SqlQuery::Select(s)),
        map(deletion, |d| SqlQuery::Delete(d)),
        map(updating, |u| SqlQuery::Update(u)),
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

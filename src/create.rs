use nom::character::complete::{multispace1};
use std::fmt;
use std::str;

use column::{Column, ColumnSpecification};
use common::{
    sql_identifier, statement_terminator, TableKey,
};
use compound_select::{compound_selection, CompoundSelectStatement};
//use create_table_options::table_options;
use keywords::escape_if_keyword;
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case};
use nom::combinator::{map};
use nom::sequence::{tuple};
use nom::IResult;
use select::{nested_selection, SelectStatement};
use table::Table;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateTableStatement {
    pub table: Table,
    pub fields: Vec<ColumnSpecification>,
    pub keys: Option<Vec<TableKey>>,
}

impl fmt::Display for CreateTableStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CREATE TABLE {} ", escape_if_keyword(&self.table.name))?;
        write!(f, "(")?;
        write!(
            f,
            "{}",
            self.fields
                .iter()
                .map(|field| format!("{}", field))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(ref keys) = self.keys {
            write!(
                f,
                ", {}",
                keys.iter()
                    .map(|key| format!("{}", key))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SelectSpecification {
    Compound(CompoundSelectStatement),
    Simple(SelectStatement),
}

impl fmt::Display for SelectSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SelectSpecification::Compound(ref csq) => write!(f, "{}", csq),
            SelectSpecification::Simple(ref sq) => write!(f, "{}", sq),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateViewStatement {
    pub name: String,
    pub fields: Vec<Column>,
    pub definition: Box<SelectSpecification>,
}

impl fmt::Display for CreateViewStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CREATE VIEW {} ", escape_if_keyword(&self.name))?;
        if !self.fields.is_empty() {
            write!(f, "(")?;
            write!(
                f,
                "{}",
                self.fields
                    .iter()
                    .map(|field| format!("{}", field))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
            write!(f, ") ")?;
        }
        write!(f, "AS ")?;
        write!(f, "{}", self.definition)
    }
}


// Parse rule for a SQL CREATE VIEW query.
pub fn view_creation(i: &[u8]) -> IResult<&[u8], CreateViewStatement> {
    let (remaining_input, (_, _, _, _, name_slice, _, _, _, def, _)) = tuple((
        tag_no_case("create"),
        multispace1,
        tag_no_case("view"),
        multispace1,
        sql_identifier,
        multispace1,
        tag_no_case("as"),
        multispace1,
        alt((
            map(compound_selection, |s| SelectSpecification::Compound(s)),
            map(nested_selection, |s| SelectSpecification::Simple(s)),
        )),
        statement_terminator,
    ))(i)?;

    let name = String::from_utf8(name_slice.to_vec()).unwrap();
    let fields = vec![]; // TODO(malte): support
    let definition = Box::new(def);

    Ok((
        remaining_input,
        CreateViewStatement {
            name,
            fields,
            definition,
        },
    ))
}


use nom::character::complete::{multispace0, multispace1};
use std::fmt;
use std::str;

use column::Column;
use common::FieldDefinitionExpression;
use common::{
    as_alias, field_definition_expr, field_list, statement_terminator, table_list, table_reference,
    unsigned_number,
};
use condition::{condition_expr, ConditionExpression};
use join::{join_operator, JoinConstraint, JoinOperator, JoinRightSide};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use order::{order_clause, OrderClause};
use table::Table;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GroupByClause {
    pub columns: Vec<Column>,
    pub having: Option<ConditionExpression>,
}

impl fmt::Display for GroupByClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GROUP BY ")?;
        write!(
            f,
            "{}",
            self.columns
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(ref having) = self.having {
            write!(f, " HAVING {}", having)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct JoinClause {
    pub operator: JoinOperator,
    pub right: JoinRightSide,
    pub constraint: JoinConstraint,
}

impl fmt::Display for JoinClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operator)?;
        write!(f, " {}", self.right)?;
        write!(f, " {}", self.constraint)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LimitClause {
    pub limit: u64,
    pub offset: u64,
}

impl fmt::Display for LimitClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LIMIT {}", self.limit)?;
        if self.offset > 0 {
            write!(f, " OFFSET {}", self.offset)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SelectStatement {
    pub tables: Vec<Table>,
    pub distinct: bool,
    pub fields: Vec<FieldDefinitionExpression>,
    pub join: Vec<JoinClause>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Option<GroupByClause>,
    pub order: Option<OrderClause>,
    pub limit: Option<LimitClause>,
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SELECT ")?;
        if self.distinct {
            write!(f, "DISTINCT ")?;
        }
        write!(
            f,
            "{}",
            self.fields
                .iter()
                .map(|field| format!("{}", field))
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        if self.tables.len() > 0 {
            write!(f, " FROM ")?;
            write!(
                f,
                "{}",
                self.tables
                    .iter()
                    .map(|table| format!("{}", table))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        for jc in &self.join {
            write!(f, " {}", jc)?;
        }
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        if let Some(ref group_by) = self.group_by {
            write!(f, " {}", group_by)?;
        }
        if let Some(ref order) = self.order {
            write!(f, " {}", order)?;
        }
        if let Some(ref limit) = self.limit {
            write!(f, " {}", limit)?;
        }
        Ok(())
    }
}

fn having_clause(i: &[u8]) -> IResult<&[u8], ConditionExpression> {
    let (remaining_input, (_, _, _, ce)) = tuple((
        multispace0,
        tag_no_case("having"),
        multispace1,
        condition_expr,
    ))(i)?;

    Ok((remaining_input, ce))
}

// Parse GROUP BY clause
pub fn group_by_clause(i: &[u8]) -> IResult<&[u8], GroupByClause> {
    let (remaining_input, (_, _, _, columns, having)) = tuple((
        multispace0,
        tag_no_case("group by"),
        multispace1,
        field_list,
        opt(having_clause),
    ))(i)?;

    Ok((remaining_input, GroupByClause { columns, having }))
}

fn offset(i: &[u8]) -> IResult<&[u8], u64> {
    let (remaining_input, (_, _, _, val)) = tuple((
        multispace0,
        tag_no_case("offset"),
        multispace1,
        unsigned_number,
    ))(i)?;

    Ok((remaining_input, val))
}

// Parse LIMIT clause
pub fn limit_clause(i: &[u8]) -> IResult<&[u8], LimitClause> {
    let (remaining_input, (_, _, _, limit, opt_offset)) = tuple((
        multispace0,
        tag_no_case("limit"),
        multispace1,
        unsigned_number,
        opt(offset),
    ))(i)?;
    let offset = match opt_offset {
        None => 0,
        Some(v) => v,
    };

    Ok((remaining_input, LimitClause { limit, offset }))
}

fn join_constraint(i: &[u8]) -> IResult<&[u8], JoinConstraint> {
    let using_clause = map(
        tuple((
            tag_no_case("using"),
            multispace1,
            delimited(
                terminated(tag("("), multispace0),
                field_list,
                preceded(multispace0, tag(")")),
            ),
        )),
        |t| JoinConstraint::Using(t.2),
    );
    let on_condition = alt((
        delimited(
            terminated(tag("("), multispace0),
            condition_expr,
            preceded(multispace0, tag(")")),
        ),
        condition_expr,
    ));
    let on_clause = map(tuple((tag_no_case("on"), multispace1, on_condition)), |t| {
        JoinConstraint::On(t.2)
    });

    alt((using_clause, on_clause))(i)
}

// Parse JOIN clause
fn join_clause(i: &[u8]) -> IResult<&[u8], JoinClause> {
    let (remaining_input, (_, _natural, operator, _, right, _, constraint)) = tuple((
        multispace0,
        opt(terminated(tag_no_case("natural"), multispace1)),
        join_operator,
        multispace1,
        join_rhs,
        multispace1,
        join_constraint,
    ))(i)?;

    Ok((
        remaining_input,
        JoinClause {
            operator,
            right,
            constraint,
        },
    ))
}

fn join_rhs(i: &[u8]) -> IResult<&[u8], JoinRightSide> {
    let nested_select = map(
        tuple((
            delimited(tag("("), nested_selection, tag(")")),
            opt(as_alias),
        )),
        |t| JoinRightSide::NestedSelect(Box::new(t.0), t.1.map(String::from)),
    );
    let nested_join = map(delimited(tag("("), join_clause, tag(")")), |nj| {
        JoinRightSide::NestedJoin(Box::new(nj))
    });
    let table = map(table_reference, |t| JoinRightSide::Table(t));
    let tables = map(delimited(tag("("), table_list, tag(")")), |tables| {
        JoinRightSide::Tables(tables)
    });
    alt((nested_select, nested_join, table, tables))(i)
}

// Parse WHERE clause of a selection
pub fn where_clause(i: &[u8]) -> IResult<&[u8], ConditionExpression> {
    let (remaining_input, (_, _, _, where_condition)) = tuple((
        multispace0,
        tag_no_case("where"),
        multispace1,
        condition_expr,
    ))(i)?;

    Ok((remaining_input, where_condition))
}

// Parse rule for a SQL selection query.
pub fn selection(i: &[u8]) -> IResult<&[u8], SelectStatement> {
    terminated(nested_selection, statement_terminator)(i)
}

pub fn nested_selection(i: &[u8]) -> IResult<&[u8], SelectStatement> {
    let (
        remaining_input,
        (_, _, distinct, _, fields, _, tables, join, where_clause, group_by, order, limit),
    ) = tuple((
        tag_no_case("select"),
        multispace1,
        opt(tag_no_case("distinct")),
        multispace0,
        field_definition_expr,
        delimited(multispace0, tag_no_case("from"), multispace0),
        table_list,
        many0(join_clause),
        opt(where_clause),
        opt(group_by_clause),
        opt(order_clause),
        opt(limit_clause),
    ))(i)?;
    Ok((
        remaining_input,
        SelectStatement {
            tables,
            distinct: distinct.is_some(),
            fields,
            join,
            where_clause,
            group_by,
            order,
            limit,
        },
    ))
}

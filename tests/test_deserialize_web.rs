use std::collections::btree_set::BTreeSet;
use std::iter::FromIterator;

use serde::Deserialize;
use serde_querystring::from_str;

macro_rules! set {
    () => (
        BTreeSet::new()
    );
    ($($x:expr),+ $(,)?) => (
        BTreeSet::from_iter(vec![$($x) ,*])
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct Query<T> {
    start: Option<usize>,
    end: Option<usize>,

    filter: Option<T>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Filter {
    ids: Option<BTreeSet<usize>>,
    name: Option<BTreeSet<StringFilter>>,
    age: Option<BTreeSet<NumberFilter>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
enum StringFilter {
    #[serde(rename = "none")]
    Empty,
    #[serde(rename = "eq")]
    Equal(String),
    #[serde(rename = "neq")]
    NotEqual(String),
    #[serde(rename = "has")]
    Contains(String),
    #[serde(rename = "len")]
    Len(BTreeSet<NumberFilter>),
}
use StringFilter::*;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum NumberFilter {
    Lt(usize),
    Lte(usize),
    Gt(usize),
    Gte(usize),
    Eq(usize),
    Neq(usize),
}
use NumberFilter::*;

#[test]
fn deserialize() {
    let input = "filter[name][]=none&filter[name][][eq]=john&filter[name][][neq]=doe&filter[name][][has]=Par&\
    filter[name][a][len][][lt]=20&filter[name][a][len][][neq]=10&filter[name][b][len][][gte]=20&\
    filter[name][b][len][b][lt]=40&filter[age][][lt]=20&filter[age][][gte]=10&filter[age][][neq]=15&\
    filter[ids]=10,12,14,1337";

    let res = Ok(Query {
        start: None,
        end: None,
        filter: Some(Filter {
            ids: Some(set![10, 12, 14, 1337]),
            name: Some(set![
                Empty,
                Equal("john".to_string()),
                NotEqual("doe".to_string()),
                Contains("Par".to_string()),
                Len(set![Lt(20), Neq(10)]),
                Len(set![Gte(20), Lt(40)]),
            ]),
            age: Some(set![Lt(20), Gte(10), Neq(15)]),
        }),
    });

    assert_eq!(res, from_str(input));
}

#[test]
fn deserialize_simple() {
    assert_eq!(
        from_str(""),
        Ok(Query::<Filter> {
            start: None,
            end: None,
            filter: None
        })
    );
    assert_eq!(
        from_str("start=10"),
        Ok(Query::<Filter> {
            start: Some(10),
            end: None,
            filter: None
        })
    );
    assert_eq!(
        from_str("end=10"),
        Ok(Query::<Filter> {
            start: None,
            end: Some(10),
            filter: None
        })
    );
    assert_eq!(
        from_str("start=10&end=20"),
        Ok(Query::<Filter> {
            start: Some(10),
            end: Some(20),
            filter: None
        })
    );
}

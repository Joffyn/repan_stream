#[cfg(feature = "ssr")]
use rusqlite::types::FromSql;
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Error, Result, Row, ToSql};
#[cfg(feature = "ssr")]
use std::fmt;
#[cfg(feature = "ssr")]
use std::path::Path;

use leptos::logging::log;
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "/home/joffy/Work/repan_stream/";
//const DB_PATH: &str = "D:\\dev\\audio-stream\\";

//#[derive(Serialize, Deserialize, Debug)]
//pub struct Jam
//{
//    pub date: String,
//    pub path: String,
//    pub tracks: Vec<String>,
//}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JamQueryResult<T> {
    pub id: i64,
    pub data: T,
}

#[derive(Debug)]
pub enum QueryTarget {
    Date,
    Path,
    Track(i64),
}

#[derive(Debug)]
pub enum QueryAmount {
    All,
    One(String),
    Month(String),
    MonthDays(String),
    Day(String),
}

#[derive(Debug)]
#[cfg(feature = "ssr")]
pub enum DatabaseError {
    AlreadyExists,
}

#[cfg(feature = "ssr")]
impl fmt::Display for DatabaseError {
    #[cfg(feature = "ssr")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::AlreadyExists => write!(f, "Item already exists"),
        }
    }
}

#[cfg(feature = "ssr")]
impl std::error::Error for DatabaseError {}

#[cfg(feature = "ssr")]
impl From<rusqlite::Error> for DatabaseError {
    #[cfg(feature = "ssr")]
    fn from(_: rusqlite::Error) -> Self {
        todo!()
    }
}
#[cfg(feature = "ssr")]
pub trait FromRow: Sized {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self>;
}
#[cfg(feature = "ssr")]
impl FromRow for i64 {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get(0)
    }
}

#[cfg(feature = "ssr")]
impl FromRow for u64 {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get(0)
    }
}
//#[cfg(feature = "ssr")]
//impl FromRow for i32 {
//    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
//        row.get(0)
//    }
//}
#[cfg(feature = "ssr")]
impl FromRow for String {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get(0)
    }
}
#[cfg(feature = "ssr")]
impl FromRow for (String, String) {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok((row.get(0)?, row.get(1)?))
    }
}

#[cfg(feature = "ssr")]
pub struct Database {
    conn: Connection,
}

#[cfg(feature = "ssr")]
impl Database {
    #[cfg(feature = "ssr")]
    fn new(conn: Connection) -> Self {
        Database { conn }
    }
    //#[cfg(feature = "ssr")]
    //pub fn query_two(&mut self) -> Result<Vec<JamQueryResult<(String, String)>>, rusqlite::Error> {

    //    let sql = "SELECT date, path, id FROM jams WHERE jam_id = ?2".to_string();
    //    let mut stmt = self.conn.prepare(&sql)?;
    //    stmt.query(params)


    //    Ok(())
    //}

    #[cfg(feature = "ssr")]
    pub fn query<T>(
        &mut self,
        target: QueryTarget,
        amount: QueryAmount,
    ) -> Result<Vec<JamQueryResult<T>>, rusqlite::Error>
    where
        T: FromSql
    {
        let (sql, params): (String, Vec<rusqlite::types::Value>) = match (target, amount)
        {
            (QueryTarget::Date, QueryAmount::All) => 
            (
                "SELECT date, id FROM jams".to_string(),
                vec![],
            ),
            (QueryTarget::Date, QueryAmount::One(path)) =>
            (
                "SELECT date, id FROM jams WHERE path = ?1".to_string(),
                vec![path.into()],
            ),
            (QueryTarget::Date, QueryAmount::Month(yearmonth)) =>
            (
                "SELECT date, id FROM jams WHERE SUBSTR(date, 1, 4) = ?1;".to_string(),
                vec![yearmonth.into()],
            ),
            (QueryTarget::Date, QueryAmount::MonthDays(yearmonth)) =>
            (
                "SELECT CAST(SUBSTR(date, 5, 2) AS INTEGER) AS day, id FROM jams WHERE SUBSTR(date, 1, 4) = ?1;".to_string(),
                vec![yearmonth.into()],
            ),
            (QueryTarget::Date, QueryAmount::Day(yearmonthday)) =>
            (
                "SELECT date, id FROM jams WHERE SUBSTR(date, 1, 6) = ?1;".to_string(),
                vec![yearmonthday.into()],
            ),
            (QueryTarget::Path, QueryAmount::All) =>
            (
                "SELECT path, id FROM jams".to_string(),
                vec![],
            ),
            (QueryTarget::Path, QueryAmount::One(date)) =>
            (
                "SELECT path, id FROM jams WHERE date = ?1".to_string(),
                vec![date.into()],
            ),
            (QueryTarget::Track(jam_id), QueryAmount::All) =>
            (
                "SELECT track, id FROM tracks WHERE jam_id = ?1".to_string(),
                vec![jam_id.into()],
            ),
            (QueryTarget::Track(jam_id), QueryAmount::One(stem)) =>
            (
                "SELECT track, id FROM tracks WHERE jam_id = ?1 AND track = ?2".to_string(),
                vec![jam_id.into(), stem.into()],
            ),
            _ => unimplemented!("Not a valid query"),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn ToSql> = params.iter().map(|v| v as &dyn ToSql).collect();
        let rows = stmt.query_map(&*params_refs, |row| {
            //let data = T::from_row(row)?;
            //let id: i64 = row.get("id")?;
            Ok(JamQueryResult {
                data: row.get(0)?,
                id: row.get(1)?,
            })
        })?;
        let results: Result<Vec<_>, _> = rows.collect();
        results
    }
}
#[cfg(feature = "ssr")]
pub fn get_database() -> Result<Database, Error> {
    let full_path = DB_PATH.to_string() + "jams.db";
    let db_path = Path::new(&full_path);

    if !db_path.is_file() {
        return Err(rusqlite::Error::InvalidPath(db_path.to_path_buf()));
    }
    let conn = Connection::open(&full_path)?;
    Ok(Database { conn })
}

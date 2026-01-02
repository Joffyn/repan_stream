use leptos::logging::log;
use leptos::prelude::*;
use leptos::server;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "ssr")]
use crate::backend::database::get_database;
use crate::backend::database::QueryType;
use crate::backend::database::{JamQueryResult, QueryAmount, QueryTarget};

//pub type MultiQuery = Vec<Vec<JamQueryResult<String>>>;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MultiQuery {
    pub id: Option<i64>,
    pub date: Option<JamQueryResult<String>>,
    pub path: Option<JamQueryResult<String>>,
    pub tracks: Option<Vec<JamQueryResult<String>>>,
}

#[server(GetTracksAndPath)]
pub async fn get_tracks_and_path(id: i64) -> Result<MultiQuery, ServerFnError> {
    let mut db = get_database()?;

    let tracks = db.query(QueryTarget::Track(id), QueryAmount::All)?;
    let path = db
        .query(QueryTarget::Path, QueryAmount::One(QueryType::FromID(id)))?
        .first()
        .unwrap()
        .to_owned();

    Ok(MultiQuery {
        id: Some(id),
        date: None,
        path: Some(path),
        tracks: Some(tracks),
    })
}

#[server(GetAllJams)]
pub async fn get_all_jams() -> Result<Vec<JamQueryResult<String>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
    Ok(res)
}
#[server(GetAllJamsFromMonth)]
pub async fn get_all_jams_month(
    yearmonth: String,
) -> Result<Vec<JamQueryResult<String>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::Month(yearmonth))?;
    Ok(res)
}
#[server(GetAllJamsFromMonthAsDays)]
pub async fn get_all_days_with_jams(
    yearmonth: String,
) -> Result<Vec<JamQueryResult<u32>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::MonthDays(yearmonth))?;
    Ok(res)
}

#[server(GetAllJamsFromDay)]
pub async fn get_all_jams_from_day(
    ymd: String,
) -> Result<Vec<JamQueryResult<String>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::Day(ymd))?;
    Ok(res)
}
#[server(GetJam)]
pub async fn get_jam(id: i64) -> Result<MultiQuery, ServerFnError> {
    let mut db = get_database()?;

    let tracks = db.query(QueryTarget::Track(id), QueryAmount::All)?;
    let path = db.query(QueryTarget::Path, QueryAmount::One(QueryType::FromID(id)))?;
    let date = db.query(QueryTarget::Date, QueryAmount::One(QueryType::FromID(id)))?;

    if path.is_empty() || date.is_empty() {
        return Err(ServerFnError::ServerError("No path or empty".to_string()));
    }
    let path = path.first().unwrap().to_owned();
    let date = date.first().unwrap().to_owned();

    Ok(MultiQuery {
        id: Some(id),
        date: Some(date),
        path: Some(path),
        tracks: Some(tracks),
    })
}

#[server(GetJamPath)]
pub async fn get_jam_path(jam_date: String) -> Result<JamQueryResult<String>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(
        QueryTarget::Path,
        QueryAmount::One(QueryType::FromDate(jam_date)),
    )?;
    Ok(res.first().expect("At least one jam to exist").clone())
}
#[server(GetTracks)]
pub async fn get_tracks(id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Track(id), QueryAmount::All)?;
    Ok(res)
}
#[server(GetTrackList)]
pub async fn get_track_list(jam_id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError> {
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Track(jam_id), QueryAmount::All)?;
    Ok(res)
}
//#[server(GetFullJam)]
//async fn get_full_jam() -> Result<Vec<JamQueryResult>, ServerFnError>
//{
//    let mut db = database::get_database("/home/joffy/audio-stream/")?;
//
//    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
//
//    let tracks = db.query(QueryTarget::Track(), amount)
//    Ok(res)
//}

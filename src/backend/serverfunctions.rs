#[cfg(feature = "ssr")]
use leptos::prelude::*;

use crate::backend::database::get_database;
#[cfg(feature = "ssr")]
use crate::backend::database::{QueryAmount, JamQueryResult, QueryTarget};


#[server(GetAllJams)]
pub async fn get_all_jams() -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
    Ok(res)
}
#[server(GetAllJamsFromMonth)]
pub async fn get_all_jams_month(yearmonth: String) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::Month(yearmonth))?;
    Ok(res)
}
#[server(GetAllJamsFromMonthAsDays)]
pub async fn get_all_days_with_jams(yearmonth: String)
-> Result<Vec<JamQueryResult<u32>>, ServerFnError>
{
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::MonthDays(yearmonth))?;
    Ok(res)
}

#[server(GetAllJamsFromDay)]
pub async fn get_all_jams_from_day(ymd: String) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{

    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::Day(ymd))?;
    Ok(res)
}
#[server(GetJam)]
pub async fn get_jam() -> Result<JamQueryResult<String>, ServerFnError>
{
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
    Ok(res.first().expect("At least one jam to exist").clone())
}
#[server(GetTracks)]
pub async fn get_tracks(id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = get_database()?;

    let res = db.query(QueryTarget::Track(id), QueryAmount::All)?;
    Ok(res)
}
#[server(GetTrackList)]
pub async fn get_track_list(jam_id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
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


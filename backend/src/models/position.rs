use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppConfig, crud_delete, crud_delete_all, crud_read, crud_update, crud_use,
    errors::ServerError, models::user::User, schema::positions, schema::positions::dsl::*,
};

macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self
        }
    };
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    AsChangeset,
    Identifiable,
    Associations,
)]
#[table_name = "positions"]
#[belongs_to(User)]
pub struct Position {
    pub id: i32,
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub source: String,
    pub battery_level: i32,
    pub sport_mode: bool,
    pub time: i64,
}

impl NewPosition {
    trim!();
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "positions"]
pub struct NewPosition {
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default = "default_source")]
    pub source: String,
    pub battery_level: i32,
    pub sport_mode: bool,
    #[serde(default = "now")]
    pub time: i64,
}
fn default_source() -> String {
    "GPS".to_string()
}
fn now() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(n) => std::time::Duration::as_millis(&n)
            .try_into()
            .unwrap_or_default(),
        Err(_) => 0,
    }
}
impl Position {
    trim!();
}

crud_use!();

macro_rules! check_close_timestamp {
    ($hm:tt, $o:tt) => {
        // ignore data push if there is already a recorded position in the same second
        match $hm.get(&$o.user_id) {
            Some(value) => {
                if $o.time >= *value - 1000 && $o.time <= *value + 1000 {
                    return Ok(HttpResponse::Conflict()
                        .body("there is already a recorded position in the same second"));
                }
            }
            None => (),
        };
    };
}

macro_rules! update_last_timestamp {
    ($hm:tt, $created_o:tt) => {
        let t = $hm.entry($created_o.user_id).or_insert($created_o.time);
        *t = $created_o.time;
    };
}

macro_rules! delete_old_positions {
    ($connection:tt) => {
        // keep only the positions of the last 24 hours
        diesel::delete(positions)
            .filter(time.le(now() - 24 * 60 * 60 * 1000))
            .execute(&$connection)?;
    };
}

#[post("")]
pub async fn create(
    pool: web::Data<DbPool>,
    o: web::Json<NewPosition>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, ServerError> {
    let mut hm = cfg.user_last_update.lock().await;
    check_close_timestamp!(hm, o);
    let conn = pool.get()?;
    match web::block(move || {
        // Check that parent for our object exists
        crate::schema::users::dsl::users
            .find(o.user_id)
            .first::<User>(&conn)?;
        delete_old_positions!(conn);
        diesel::insert_into(positions)
            .values(o.clone().trim())
            .execute(&conn)?;
        let o = positions.order(id.desc()).first::<Position>(&conn)?;
        Ok(o)
    })
    .await?
    {
        Ok(created_o) => {
            update_last_timestamp!(hm, created_o);
            Ok(HttpResponse::Created().json(created_o))
        }
        Err(e) => match e {
            diesel::result::Error::DatabaseError(_, _) => {
                Ok(HttpResponse::Conflict().body(format!("{}", e)))
            }
            diesel::result::Error::NotFound => Ok(HttpResponse::NotFound().body("Item not found")),
            _ => Ok(HttpResponse::InternalServerError().body("")),
        },
    }
}

crud_read!(Position, positions);

#[derive(Deserialize)]
pub struct Params {
    user_id: i32,
}

#[get("")]
pub async fn read_filter(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServerError> {
    let conn = pool.get()?;
    let params = web::Query::<Params>::from_query(req.query_string());
    let object;
    match params {
        Ok(p) => {
            object = web::block(move || {
                positions
                    .filter(user_id.eq(p.user_id))
                    .order(id.asc())
                    .load::<Position>(&conn)
            })
            .await?;
        }
        Err(_) => {
            let conn = pool.get()?;
            object = web::block(move || positions.order(id.asc()).load::<Position>(&conn)).await?;
        }
    }
    if let Ok(object) = object {
        Ok(HttpResponse::Ok().json(object))
    } else {
        let res = HttpResponse::NotFound().body(format!("No objects found"));
        Ok(res)
    }
}

crud_update!(Position, positions, User, users, user_id);
crud_delete_all!(Position, positions);
crud_delete!(Position, positions);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellId {
    pub network_type: String,
    pub mcc: String,
    pub mnc: String,
    pub cid: i64,
    pub lac: i32,
    pub lat: i64,
    pub long: i64,
    pub battery_level: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCellIdResponse {
    pub lat: f64,
    pub lon: f64,
    pub mcc: i32,
    pub mnc: i32,
    pub lac: i32,
    pub cellid: i32,
    pub average_signal_strength: i32,
    pub range: i32,
    pub samples: i32,
    pub changeable: i32,
    pub radio: String,
    pub rnc: i32,
    pub cid: i64,
    pub tac: i32,
    pub sid: i32,
    pub nid: i32,
    pub bid: i32,
    #[serde(skip)]
    pub message: String,
}

#[post("/cid/{uid}")]
pub async fn create_from_cid(
    pool: web::Data<DbPool>,
    uid: web::Path<i32>,
    cell_id: web::Json<CellId>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, ServerError> {
    let mut hm = cfg.user_last_update.lock().await;
    let mut o = NewPosition {
        user_id: *uid,
        latitude: 0.0,
        longitude: 0.0,
        source: format!("Cell Id ({})", cell_id.network_type),
        time: now(),
        battery_level: cell_id.battery_level,
        sport_mode: false,
    };
    check_close_timestamp!(hm, o);
    if cell_id.lat != -1 {
        o.latitude = (cell_id.lat / 1296000) as f64;
        o.longitude = (cell_id.long / 2592000) as f64;
    } else {
        // Get the cell location from open Cell Id database
        let ocid_resp = get_resp(&cell_id, &cfg.open_cell_id_api_key).await?;
        // Create position from those informations
        o.latitude = ocid_resp.lat;
        o.longitude = ocid_resp.lon;
    };
    let conn = pool.get()?;
    match web::block(move || {
        // Check that parent for our object exists
        crate::schema::users::dsl::users
            .find(o.user_id)
            .first::<User>(&conn)?;
        delete_old_positions!(conn);
        diesel::insert_into(positions)
            .values(o.clone().trim())
            .execute(&conn)?;
        let o = positions.order(id.desc()).first::<Position>(&conn)?;
        Ok(o)
    })
    .await?
    {
        Ok(created_o) => {
            update_last_timestamp!(hm, created_o);
            Ok(HttpResponse::Created().json(created_o))
        }
        Err(e) => match e {
            diesel::result::Error::DatabaseError(_, _) => {
                Ok(HttpResponse::Conflict().body(format!("{}", e)))
            }
            diesel::result::Error::NotFound => Ok(HttpResponse::NotFound().body("Item not found")),
            _ => Ok(HttpResponse::InternalServerError().body("")),
        },
    }
}

async fn get_resp(cell_id: &CellId, api_key: &str) -> Result<OpenCellIdResponse, ServerError> {
    // Request latitude and longitude from OpenCellId
    let url = format!(
        "https://opencellid.org/cell/get?key={}&mcc={}&mnc={}&lac={}&cellid={}&format=json",
        api_key, cell_id.mcc, cell_id.mnc, cell_id.lac, cell_id.cid
    );
    log::info!(
        "Creating position from open cell id database with url {}",
        url
    );
    match reqwest::get(url).await {
        Ok(res) => match res.json().await {
            Ok(v) => Ok(v),
            Err(e) => Err(ServerError::Other(format!(
                "Cell not found in Open Cell ID Database: {}",
                e
            ))),
        },
        Err(e) => Err(ServerError::Other(format!(
            "Open Cell ID did not respond: {}",
            e
        ))),
    }
}

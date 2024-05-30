use crate::models::position_ws::{Message, WebSocketsState};
use actix_web::HttpRequest;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppConfig, crud_delete, crud_delete_all, crud_read, crud_update, crud_use,
    errors::ServerError, models::user::User, schema::positions, schema::positions::dsl::*,
};

const MINIMUM_TIME_GAP: i64 = 1000;

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
#[diesel(table_name = positions, belongs_to(User))]
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

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Default, PartialEq)]
#[diesel(table_name = positions)]
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

fn filter_positions(
    pos_vec: Vec<NewPosition>,
    reference: Option<i64>,
    uid: Option<i32>,
) -> Vec<NewPosition> {
    if let Some(filter_user_id) = uid {
        let mut filtered_positions = Vec::new();
        let mut last_time = None;
        for position in pos_vec {
            if position.user_id == filter_user_id {
                // Check if the time difference with the reference is greater than or equal to MINIMUM_TIME_GAP
                if reference.is_none()
                    || (position.time - reference.unwrap()).abs() >= MINIMUM_TIME_GAP
                {
                    // Check if there's a preceding position and its time difference is greater than MINIMUM_TIME_GAP
                    let pos_time = position.time;
                    if let Some(prev_time) = last_time {
                        let diff: i64 = pos_time - prev_time;
                        if diff.abs() >= MINIMUM_TIME_GAP {
                            filtered_positions.push(position);
                        }
                    } else {
                        // If there's no preceding position, add the current one
                        filtered_positions.push(position);
                    }
                    last_time = Some(pos_time);
                }
            }
        }
        return filtered_positions;
    }

    pos_vec // Return the original vector untouched
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
            .execute(&mut $connection)?;
    };
}

#[post("")]
pub async fn create(
    pool: web::Data<DbPool>,
    o: web::Json<Vec<NewPosition>>,
    cfg: web::Data<AppConfig>,
    ws_data: web::Data<WebSocketsState>,
) -> Result<HttpResponse, ServerError> {
    let mut hm = cfg.user_last_update.lock().await;
    // Filter the positions : remove those that have a timestamp too close to the last update or too close together
    let uid = o[0].user_id;
    let o = filter_positions(o.to_owned(), hm.get(&uid).copied(), Some(uid));
    if o.is_empty() {
        return Ok(HttpResponse::Conflict()
            .body("there is already a recorded position in the same second"));
    }
    let mut conn = pool.get()?;
    match web::block(move || {
        // Check that parent for our object exists
        crate::schema::users::dsl::users
            .find(o[0].user_id)
            .first::<User>(&mut conn)?;
        delete_old_positions!(conn);
        diesel::insert_into(positions)
            .values(&(*o))
            .execute(&mut conn)?;
        let o = positions
            .filter(user_id.eq(uid))
            .order(time.desc())
            .first::<Position>(&mut conn)?;
        Ok(o)
    })
    .await?
    {
        Ok(created_o) => {
            update_last_timestamp!(hm, created_o);
            let ws_actors = ws_data.ws_actors.lock().unwrap();
            debug!("sending position to websocket actors: {:?}", ws_actors);
            ws_actors
                .iter()
                .filter(|e| e.user_id == created_o.user_id.try_into().unwrap_or(0))
                .for_each(|element| {
                    element.addr.do_send(Message(created_o.clone()));
                });
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
    let mut conn = pool.get()?;
    let params = web::Query::<Params>::from_query(req.query_string());
    let object = match params {
        Ok(p) => {
            web::block(move || {
                positions
                    .filter(user_id.eq(p.user_id))
                    .order(id.asc())
                    .load::<Position>(&mut conn)
            })
            .await?
        }
        Err(_) => {
            /*let mut conn = pool.get()?;
            web::block(move || positions.order(id.asc()).load::<Position>(&mut conn)).await?*/
            let res = HttpResponse::NotFound().body("No user_id provided in query");
            return Ok(res);
        }
    };
    if let Ok(object) = object {
        Ok(HttpResponse::Ok().json(object))
    } else {
        let res = HttpResponse::NotFound().body("No objects found");
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
    if let Some(last_update) = hm.get(&o.user_id) {
        if (o.time - last_update).abs() < MINIMUM_TIME_GAP {
            return Ok(HttpResponse::Conflict()
                .body("there is already a recorded position in the same second"));
        }
    }
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
    let mut conn = pool.get()?;
    match web::block(move || {
        // Check that parent for our object exists
        crate::schema::users::dsl::users
            .find(o.user_id)
            .first::<User>(&mut conn)?;
        delete_old_positions!(conn);
        diesel::insert_into(positions)
            .values(o)
            .execute(&mut conn)?;
        let o = positions.order(id.desc()).first::<Position>(&mut conn)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_positions() {
        let reference = Some(2500);
        let vec_pos = vec![
            NewPosition {
                user_id: 1,
                time: 2000,
                ..Default::default()
            },
            NewPosition {
                user_id: 2,
                time: 2000,
                ..Default::default()
            },
            NewPosition {
                user_id: 1,
                time: 2500,
                ..Default::default()
            },
            NewPosition {
                user_id: 2,
                time: 2500,
                ..Default::default()
            },
            NewPosition {
                user_id: 1,
                time: 4000,
                ..Default::default()
            },
            NewPosition {
                user_id: 2,
                time: 4000,
                ..Default::default()
            },
            NewPosition {
                user_id: 1,
                time: 5500,
                ..Default::default()
            },
            NewPosition {
                user_id: 2,
                time: 5500,
                ..Default::default()
            },
        ];

        // Test case 1: Filtering by user_id 2 and reference 2500
        let uid = Some(2);
        let filtered_positions_1 = filter_positions(vec_pos.clone(), reference, uid);
        assert_eq!(
            filtered_positions_1,
            vec![
                NewPosition {
                    user_id: 2,
                    time: 4000,
                    ..Default::default()
                },
                NewPosition {
                    user_id: 2,
                    time: 5500,
                    ..Default::default()
                },
            ]
        );

        // Test case 2: Filtering by user_id 3 and reference 5000
        let uid = Some(3);
        let filtered_positions_2 = filter_positions(vec_pos.clone(), reference, uid);
        assert_eq!(filtered_positions_2, vec![]);

        // Test case 3: No filtering (uid = None)
        let uid = None;
        let filtered_positions_3 = filter_positions(vec_pos.clone(), reference, uid);
        assert_eq!(filtered_positions_3, vec_pos);

        // Test case 4: Empty positions vector
        let empty_positions = Vec::new();
        let uid = Some(2);
        let filtered_positions_4 = filter_positions(empty_positions.clone(), reference, uid);
        assert_eq!(filtered_positions_4, Vec::new());

        // Test case 5: Empty positions vector (uid = None)
        let uid = None;
        let filtered_positions_5 = filter_positions(empty_positions.clone(), reference, uid);
        assert_eq!(filtered_positions_5, empty_positions);

        // Test case 6: Filtering by user_id 2 and no reference
        let uid = Some(2);
        let reference = None;
        let filtered_positions_1 = filter_positions(vec_pos.clone(), reference, uid);
        assert_eq!(
            filtered_positions_1,
            vec![
                NewPosition {
                    user_id: 2,
                    time: 2000,
                    ..Default::default()
                },
                NewPosition {
                    user_id: 2,
                    time: 4000,
                    ..Default::default()
                },
                NewPosition {
                    user_id: 2,
                    time: 5500,
                    ..Default::default()
                },
            ]
        );
    }
}

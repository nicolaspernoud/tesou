use crate::app::AppConfig;
use crate::errors::ServerError;
use actix_web::{HttpResponse, post, web};

#[post("/toggle/{user_id}")]
pub async fn toggle_sport_mode(
    cfg: web::Data<AppConfig>,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, ServerError> {
    let mut sport_mode_toggle_users = cfg.sport_mode_toggle_users.lock().await;
    let uid = user_id.into_inner();
    if !sport_mode_toggle_users.contains(&uid) {
        sport_mode_toggle_users.push(uid);
    }
    Ok(HttpResponse::Ok().body(format!("User {} added to sport mode toggle list", uid)))
}

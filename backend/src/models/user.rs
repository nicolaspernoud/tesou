use serde::{Deserialize, Serialize};

use crate::{
    crud_create, crud_delete, crud_delete_all, crud_update, crud_use, errors::ServerError,
    schema::users,
};

macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self.name = self.name.trim().to_string();
            self.surname = self.surname.trim().to_string();
            self
        }
    };
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset, Identifiable,
)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub surname: String,
}
impl User {
    trim!();
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub surname: String,
}
impl NewUser {
    trim!();
}

crud_use!();
crud_create!(NewUser, User, users,);

#[derive(Serialize)]
struct ReturnedUser {
    #[serde(flatten)]
    user: User,
    switching_mode: bool,
}

#[get("")]
pub async fn read_all(
    pool: web::Data<DbPool>,
    cfg: web::Data<crate::app::AppConfig>,
) -> Result<HttpResponse, ServerError> {
    let mut conn = pool.get()?;
    let users = web::block(move || {
        use crate::schema::users::dsl::*;
        users.order(name.asc()).load::<User>(&mut conn)
    })
    .await??;

    let sport_mode_toggle_users = cfg.sport_mode_toggle_users.lock().await;
    let returned_users: Vec<ReturnedUser> = users
        .into_iter()
        .map(|user| {
            let uid = user.id;
            ReturnedUser {
                user,
                switching_mode: sport_mode_toggle_users.contains(&uid),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(returned_users))
}

#[get("/{oid}")]
pub async fn read(
    pool: web::Data<DbPool>,
    oid: web::Path<i32>,
    cfg: web::Data<crate::app::AppConfig>,
) -> Result<HttpResponse, ServerError> {
    let mut conn = pool.get()?;
    let user = web::block(move || {
        use crate::schema::users::dsl::*;
        users.filter(id.eq(*oid)).first::<User>(&mut conn)
    })
    .await??;
    let sport_mode_toggle_users = cfg.sport_mode_toggle_users.lock().await;
    let uid = user.id;
    let returned_user = ReturnedUser {
        user,
        switching_mode: sport_mode_toggle_users.contains(&uid),
    };
    Ok(HttpResponse::Ok().json(returned_user))
}

crud_update!(User, users,);
crud_delete!(User, users);
crud_delete_all!(User, users);

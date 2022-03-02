#[macro_export]
macro_rules! crud_use {
    () => {
        use actix_web::{delete, get, post, put, web, HttpResponse};
        use diesel::prelude::*;
        use diesel::r2d2::ConnectionManager;
        type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
    };
}

#[macro_export]
macro_rules! crud_read {
    ($model:ty, $table:tt) => {
        #[get("/{oid}")]
        pub async fn read(
            pool: web::Data<DbPool>,
            oid: web::Path<i32>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let iid = oid.clone();
            let object = web::block(move || {
                use crate::schema::$table::dsl::*;
                $table.filter(id.eq(oid.clone())).first::<$model>(&conn)
            })
            .await?;
            if let Ok(object) = object {
                Ok(HttpResponse::Ok().json(object))
            } else {
                let res =
                    HttpResponse::NotFound().body(format!("No object found with id: {}", iid));
                Ok(res)
            }
        }
    };
}

#[macro_export]
macro_rules! crud_read_all {
    ($model:ty, $table:tt) => {
        #[get("")]
        pub async fn read_all(pool: web::Data<DbPool>) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let object = web::block(move || {
                use crate::schema::$table::dsl::*;
                $table.order(name.asc()).load::<$model>(&conn)
            })
            .await?;
            if let Ok(object) = object {
                Ok(HttpResponse::Ok().json(object))
            } else {
                let res = HttpResponse::NotFound().body("No objects found");
                Ok(res)
            }
        }
    };
}

#[macro_export]
macro_rules! crud_create {
    ($inmodel:ty, $outmodel:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),* ) => {
        #[post("")]
        pub async fn create(
            pool: web::Data<DbPool>,
            o: web::Json<$inmodel>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            match web::block(move || {
                $(
                    // Check that parent for our object exists
                    crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&conn)?;
                )*
                use crate::schema::$table::dsl::*;
                diesel::insert_into($table)
                    .values(o.clone().trim())
                    .execute(&conn)?;
                let o = $table.order(id.desc()).first::<$outmodel>(&conn);
                o
            })
            .await? {
                Ok(created_o) => Ok(HttpResponse::Created().json(created_o)),
                Err(e) => match e {
                    diesel::result::Error::DatabaseError(_,_) => Ok(HttpResponse::Conflict().body(format!("{}", e))),
                    diesel::result::Error::NotFound => Ok(HttpResponse::NotFound().body("Item not found")),
                    _ => Ok(HttpResponse::InternalServerError().body("")),
                },

            }
        }
    };
}

#[macro_export]
macro_rules! crud_update {
    ($model:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),*) => {
        #[put("/{oid}")]
        pub async fn update(
            pool: web::Data<DbPool>,
            o: web::Json<$model>,
            oid: web::Path<i32>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let o_value = o.clone();
            let iid = oid.clone();
            let put_o = web::block(move || {
                $(
                    // Check that parent for our object exists
                    crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&conn)?;
                )*
                use crate::schema::$table::dsl::*;
                diesel::update($table)
                    .filter(id.eq(oid.clone()))
                    .set(o_value.trim())
                    .execute(&conn)?;
                $table.filter(id.eq(oid.clone())).first::<$model>(&conn)
            })
            .await?;
            if let Ok(put_o) = put_o {
                Ok(HttpResponse::Ok().json(put_o))
            } else {
                let res =
                    HttpResponse::NotFound().body(format!("No object found with id: {}", iid));
                Ok(res)
            }
        }
    };
}

#[macro_export]
macro_rules! crud_delete {
    ($model:ty, $table:tt) => {
        #[delete("/{oid}")]
        pub async fn delete(
            pool: web::Data<DbPool>,
            oid: web::Path<i32>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let oid = *oid;
            let iid = oid.clone();
            let d = web::block(move || {
                use crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table).filter(id.eq(oid)).execute(&conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound),
                    _ => Ok(deleted),
                }
            })
            .await?;
            if let Ok(_) = d {
                Ok(HttpResponse::Ok().body(format!("Deleted object with id: {}", oid)))
            } else {
                let res =
                    HttpResponse::NotFound().body(format!("No object found with id: {}", iid));
                Ok(res)
            }
        }
    };
}

#[macro_export]
macro_rules! crud_delete_all {
    ($model:ty, $table:tt) => {
        #[delete("")]
        pub async fn delete_all(pool: web::Data<DbPool>) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let d = web::block(move || {
                use crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table).execute(&conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound),
                    _ => Ok(deleted),
                }
            })
            .await?;
            if let Ok(_) = d {
                Ok(HttpResponse::Ok().body("Deleted all objects"))
            } else {
                let res = HttpResponse::NotFound().body("No objects found");
                Ok(res)
            }
        }
    };
}

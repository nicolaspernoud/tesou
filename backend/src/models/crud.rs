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
            let object = web::block(move || {
                use crate::schema::$table::dsl::*;
                $table.filter(id.eq(*oid)).first::<$model>(&conn)
            })
            .await??;
            Ok(HttpResponse::Ok().json(object))
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
            .await??;
            Ok(HttpResponse::Ok().json(object))
        }
    };
}

#[macro_export]
macro_rules! crud_create {
    ($inmodel:ty, $outmodel:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),* ) => {
        #[post("")]
        pub async fn create(
            pool: web::Data<DbPool>,
            mut o: web::Json<$inmodel>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let created_o: Result<$outmodel, ServerError> = web::block(move || {
                $(
                    // Check that parent for our object exists
                    crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&conn)?;
                )*
                use crate::schema::$table::dsl::*;
                o.trim();
                diesel::insert_into($table)
                    .values(&*o)
                    .execute(&conn)?;
                let o = $table.order(id.desc()).first::<$outmodel>(&conn)?;
                Ok(o)
            })
            .await?;
                Ok(HttpResponse::Created().json(created_o?))
        }
    };
}

#[macro_export]
macro_rules! crud_update {
    ($model:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),*) => {
        #[put("/{oid}")]
        pub async fn update(
            pool: web::Data<DbPool>,
            mut o: web::Json<$model>,
            oid: web::Path<i32>,
        ) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            let put_o: Result<$model, ServerError> = web::block(move || {
                $(
                    // Check that parent for our object exists
                    crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&conn)?;
                )*
                use crate::schema::$table::dsl::*;
                o.trim();

                diesel::update($table)
                    .filter(id.eq(*oid))
                    .set(&*o)
                    .execute(&conn)?;
                let o = $table.filter(id.eq(*oid)).first::<$model>(&conn)?;
                Ok(o)
            })
            .await?;
            Ok(HttpResponse::Ok().json(put_o?))
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
            web::block(move || {
                use crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table).filter(id.eq(oid)).execute(&conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound),
                    _ => Ok(deleted),
                }
            })
            .await??;
            Ok(HttpResponse::Ok().body(format!("Deleted object with id: {}", oid)))
        }
    };
}

#[macro_export]
macro_rules! crud_delete_all {
    ($model:ty, $table:tt) => {
        #[delete("")]
        pub async fn delete_all(pool: web::Data<DbPool>) -> Result<HttpResponse, ServerError> {
            let conn = pool.get()?;
            web::block(move || {
                use crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table).execute(&conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound),
                    _ => Ok(deleted),
                }
            })
            .await??;
            Ok(HttpResponse::Ok().body("Deleted all objects"))
        }
    };
}

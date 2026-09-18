#[macro_export]
macro_rules! crud_use {
    () => {
        use axum::{
            extract::{Path, State},
            http::StatusCode,
            response::{IntoResponse, Response},
            Json,
        };
        use diesel::prelude::*;
        use $crate::app::AppState;
        use $crate::errors::ServerError;
    };
}

#[macro_export]
macro_rules! crud_read {
    ($model:ty, $table:tt) => {
        pub async fn read(
            State(state): State<AppState>,
            Path(oid): Path<i32>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            let object = tokio::task::spawn_blocking(move || -> Result<$model, ServerError> {
                let mut conn = pool.get()?;
                use $crate::schema::$table::dsl::*;
                let res = $table.filter(id.eq(oid)).first::<$model>(&mut conn)?;
                Ok(res)
            })
            .await??;
            Ok((StatusCode::OK, Json(object)).into_response())
        }
    };
}

#[macro_export]
macro_rules! crud_read_all {
    ($model:ty, $table:tt) => {
        pub async fn read_all(
            State(state): State<AppState>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            let object = tokio::task::spawn_blocking(move || -> Result<Vec<$model>, ServerError> {
                let mut conn = pool.get()?;
                use $crate::schema::$table::dsl::*;
                let res = $table.order(name.asc()).load::<$model>(&mut conn)?;
                Ok(res)
            })
            .await??;
            Ok((StatusCode::OK, Json(object)).into_response())
        }
    };
}

#[macro_export]
macro_rules! crud_create {
    ($inmodel:ty, $outmodel:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),* ) => {
        pub async fn create(
            State(state): State<AppState>,
            Json(mut o): Json<$inmodel>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            let created_o: Result<$outmodel, ServerError> = tokio::task::spawn_blocking(move || -> Result<$outmodel, ServerError> {
                let mut conn = pool.get()?;
                $(
                    // Check that parent for our object exists
                    $crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&mut conn)?;
                )*
                use $crate::schema::$table::dsl::*;
                o.trim();
                diesel::insert_into($table)
                    .values(&o)
                    .execute(&mut conn)?;
                let o = $table.order(id.desc()).first::<$outmodel>(&mut conn)?;
                Ok(o)
            })
            .await?;
            Ok((StatusCode::CREATED, Json(created_o?)).into_response())
        }
    };
}

#[macro_export]
macro_rules! crud_update {
    ($model:ty, $table:tt, $( $parent_model:ty, $parent_table:tt, $parent_table_id:tt ),*) => {
        pub async fn update(
            State(state): State<AppState>,
            Path(oid): Path<i32>,
            Json(mut o): Json<$model>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            let put_o: Result<$model, ServerError> = tokio::task::spawn_blocking(move || -> Result<$model, ServerError> {
                let mut conn = pool.get()?;
                $(
                    // Check that parent for our object exists
                    $crate::schema::$parent_table::dsl::$parent_table.find(o.$parent_table_id).first::<$parent_model>(&mut conn)?;
                )*
                use $crate::schema::$table::dsl::*;
                o.trim();

                diesel::update($table)
                    .filter(id.eq(oid))
                    .set(&o)
                    .execute(&mut conn)?;
                let o = $table.filter(id.eq(oid)).first::<$model>(&mut conn)?;
                Ok(o)
            })
            .await?;
            Ok((StatusCode::OK, Json(put_o?)).into_response())
        }
    };
}

#[macro_export]
macro_rules! crud_delete {
    ($model:ty, $table:tt) => {
        pub async fn delete(
            State(state): State<AppState>,
            Path(oid): Path<i32>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            tokio::task::spawn_blocking(move || -> Result<usize, ServerError> {
                let mut conn = pool.get()?;
                use $crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table)
                    .filter(id.eq(oid))
                    .execute(&mut conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound.into()),
                    _ => Ok(deleted),
                }
            })
            .await??;
            Ok((StatusCode::OK, format!("Deleted object with id: {}", oid)).into_response())
        }
    };
}

#[macro_export]
macro_rules! crud_delete_all {
    ($model:ty, $table:tt) => {
        pub async fn delete_all(
            State(state): State<AppState>,
        ) -> Result<Response, ServerError> {
            let pool = state.pool;
            tokio::task::spawn_blocking(move || -> Result<usize, ServerError> {
                let mut conn = pool.get()?;
                use $crate::schema::$table::dsl::*;
                let deleted = diesel::delete($table).execute(&mut conn)?;
                match deleted {
                    0 => Err(diesel::result::Error::NotFound.into()),
                    _ => Ok(deleted),
                }
            })
            .await??;
            Ok((StatusCode::OK, "Deleted all objects").into_response())
        }
    };
}

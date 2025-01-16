use crate::{app::AppConfig, create_app, positions_server::PositionsServerHandle};

pub async fn user_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &actix_web::web::Data<AppConfig>,
    position_server_handle: &PositionsServerHandle,
) {
    use crate::{do_test, do_test_extract_id};
    use actix_web::{
        http::{Method, StatusCode},
        test,
    };

    let mut app = test::init_service(create_app!(pool, app_config, position_server_handle)).await;

    // Delete all the users
    let req = test::TestRequest::delete()
        .insert_header(("Authorization", "Bearer 0101"))
        .uri("/api/users")
        .to_request();
    test::call_service(&app, req).await;

    // Create a user
    let id = do_test_extract_id!(
        app,
        Method::POST,
        "/api/users",
        "{\"name\":\"  Test name  \",\"surname\":\"    Test surname       \"}",
        StatusCode::CREATED,
        "{\"id\""
    );

    // Get a user
    do_test!(
        app,
        Method::GET,
        &format!("/api/users/{}", id),
        "",
        StatusCode::OK,
        format!(
            "{{\"id\":{},\"name\":\"Test name\",\"surname\":\"Test surname\"}}",
            id
        )
    );

    // Get a non existing user
    do_test!(
        app,
        Method::GET,
        &format!("/api/users/{}", id + 1),
        "",
        StatusCode::NOT_FOUND,
        "Item not found"
    );

    // Patch the user
    do_test!(
        app,
        Method::PUT,
        &format!("/api/users/{}", id),
        &format!("{{\"id\":{}, \"name\":\"  Patched test name   \",\"surname\":\"    Patched test surname       \"}}",id),
        StatusCode::OK,
        "{\"id\""
    );

    // Delete the user
    do_test!(
        app,
        Method::DELETE,
        &format!("/api/users/{}", id),
        "",
        StatusCode::OK,
        format!("Deleted object with id: {}", id)
    );

    // Delete a non existing user
    do_test!(
        app,
        Method::DELETE,
        &format!("/api/users/{}", id + 1),
        "",
        StatusCode::NOT_FOUND,
        "Item not found"
    );

    // Delete all the users
    let req = test::TestRequest::delete()
        .insert_header(("Authorization", "Bearer 0101"))
        .uri("/api/users")
        .to_request();
    test::call_service(&app, req).await;

    // Create two users and get them all
    let id1 = do_test_extract_id!(
        app,
        Method::POST,
        "/api/users",
        "{\"name\":\"01_name\",\"surname\":\"01_description\"}",
        StatusCode::CREATED,
        "{\"id\""
    );
    let id2 = do_test_extract_id!(
        app,
        Method::POST,
        "/api/users",
        "{\"name\":\"02_name\",\"surname\":\"02_description\"}",
        StatusCode::CREATED,
        "{\"id\""
    );
    do_test!(
        app,
        Method::GET,
        "/api/users",
        "",
        StatusCode::OK,
        format!("[{{\"id\":{},\"name\":\"01_name\",\"surname\":\"01_description\"}},{{\"id\":{},\"name\":\"02_name\",\"surname\":\"02_description\"}}]", id1, id2)
    );

    // Delete all the users
    do_test!(
        app,
        Method::DELETE,
        "/api/users",
        "",
        StatusCode::OK,
        "Deleted all objects"
    );
}

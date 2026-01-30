use crate::{app::AppConfig, create_app, positions_server::PositionsServerHandle};

pub async fn toggle_sport_mode_test(
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
    do_test!(
        app,
        Method::DELETE,
        "/api/users",
        "",
        StatusCode::OK,
        "Deleted all objects"
    );

    // Create a user
    let user_id = do_test_extract_id!(
        app,
        Method::POST,
        "/api/users",
        r#"{"name":"  Test name  ","surname":"    Test surname       "}"#,
        StatusCode::CREATED,
        "{\"id\""
    );

    // Create a position with an existing user
    do_test!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );

    // Toggle sport mode for the user
    do_test!(
        app,
        Method::POST,
        &format!("/api/sport-mode/toggle/{}", user_id),
        "",
        StatusCode::OK,
        &format!("User {} added to sport mode toggle list", user_id)
    );

    // Get the user, switching_mode should be true now
    do_test!(
        app,
        Method::GET,
        &format!("/api/users/{}", user_id),
        "",
        StatusCode::OK,
        format!(
            "{{\"id\":{},\"name\":\"Test name\",\"surname\":\"Test surname\",\"switching_mode\":true}}",
            user_id
        )
    );

    // Get all users, the created user should have switching_mode true
    do_test!(
        app,
        Method::GET,
        "/api/users",
        "",
        StatusCode::OK,
        format!(
            "[{{\"id\":{},\"name\":\"Test name\",\"surname\":\"Test surname\",\"switching_mode\":true}}]",
            user_id
        )
    );

    // Wait for a second to cater for position creation rate limit
    std::thread::sleep(core::time::Duration::from_secs(1));

    // Create another position for the same user, sport_mode should be true now
    let pos_body = do_test!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );
    let pos: crate::models::position::Position = serde_json::from_str(&pos_body).unwrap();
    assert!(pos.sport_mode);

    // Get the user, switching_mode should be returned to false
    do_test!(
        app,
        Method::GET,
        &format!("/api/users/{}", user_id),
        "",
        StatusCode::OK,
        format!(
            "{{\"id\":{},\"name\":\"Test name\",\"surname\":\"Test surname\",\"switching_mode\":false}}",
            user_id
        )
    );

    // Get all users, the created user should have switching_mode false
    do_test!(
        app,
        Method::GET,
        "/api/users",
        "",
        StatusCode::OK,
        format!(
            "[{{\"id\":{},\"name\":\"Test name\",\"surname\":\"Test surname\",\"switching_mode\":false}}]",
            user_id
        )
    );

    // Delete all the positions
    do_test!(
        app,
        Method::DELETE,
        "/api/positions",
        "",
        StatusCode::OK,
        "Deleted all objects"
    );
}

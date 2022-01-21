use crate::{app::AppConfig, create_app};

pub async fn position_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: AppConfig,
) {
    use crate::{do_test, do_test_extract_id};
    use actix_web::{
        http::{Method, StatusCode},
        test,
    };

    let mut app = test::init_service(create_app!(pool, app_config)).await;

    impl std::fmt::Display for crate::models::position::Position {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "
                id: {}\n
                user_id: {}\n
                time: {}\n
                latitude: {}\n
                longitude: {}\n
                source: {}\n
                ",
                self.id, self.user_id, self.time, self.latitude, self.longitude, self.source
            )
        }
    }

    // Check that using the wrong token gives an unauthorized error
    let req = test::TestRequest::with_uri("/api/positions")
        .method(Method::GET)
        .header("Authorization", "Bearer 0102")
        .to_request();
    use actix_web::dev::Service;
    let resp = app.call(req).await;
    assert!(resp.is_err());
    assert!(resp.err().unwrap().to_string() == "Wrong token!");

    // Create a position with a non existing user
    do_test!(
        app,
        Method::POST,
        "/api/positions",
        r#"{"user_id":1,"latitude":45.74846,"longitude":4.84671,"source":"GPS"}"#,
        StatusCode::NOT_FOUND,
        "Position not found"
    );

    let user_id = do_test_extract_id!(
        app,
        Method::POST,
        "/api/users",
        r#"{"name":"  Test name  ","surname":"    Test surname       "}"#,
        StatusCode::CREATED,
        "{\"id\""
    );

    // Create a position with an existing user
    let id = do_test_extract_id!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS"}}"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );

    // Immediatly create a position with the same user (to test rate limit)
    do_test!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS"}}"#,
            user_id
        ),
        StatusCode::CONFLICT,
        "there is already a recorded position in the same second"
    );

    // Get a position
    do_test!(
        app,
        Method::GET,
        &format!("/api/positions/{}", id),
        "",
        StatusCode::OK,
        format!(
            r#"{{"id":{},"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","time":"#,
            id, user_id
        )
    );

    // Get a non existing position
    do_test!(
        app,
        Method::GET,
        &format!("/api/positions/{}", id + 1),
        "",
        StatusCode::NOT_FOUND,
        "Position not found"
    );

    // Patch the position
    do_test!(
        app,
        Method::PUT,
        &format!("/api/positions/{}", id),
        &crate::models::position::Position {
            id: id,
            user_id: user_id,
            latitude: 45.1911396,
            longitude: 5.7141747,
            source: "GPS".to_string(),
            time: 0
        },
        StatusCode::OK,
        format!(
            r#"{{"id":{},"user_id":{},"latitude":45.1911396,"longitude":5.7141747,"source":"GPS","time":"#,
            id, user_id
        )
    );

    // Delete the position
    do_test!(
        app,
        Method::DELETE,
        &format!("/api/positions/{}", id),
        "",
        StatusCode::OK,
        format!("Deleted object with id: {}", id)
    );

    // Delete a non existing position
    do_test!(
        app,
        Method::DELETE,
        &format!("/api/positions/{}", id + 1),
        "",
        StatusCode::NOT_FOUND,
        "Position not found"
    );

    // Create an old position
    do_test_extract_id!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"{{"user_id":{},"latitude":37.421998333333335,"longitude":-122.084,"source":"GPS","time":1642608103000}}"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );
    // Create a position
    let id1 = do_test_extract_id!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS"}}"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );
    // Wait for a second to cater for position creation rate limit
    std::thread::sleep(core::time::Duration::from_secs(1));
    do_test!(
        app,
        Method::POST,
        "/api/positions",
        &format!(
            r#"{{"user_id":{},"latitude":45.1911396,"longitude":5.7141747,"source":"GPS"}}"#,
            user_id
        ),
        StatusCode::CREATED,
        "{\"id\""
    );
    // Get the positions and test that the first is id1 (the old one beeing deleted)
    do_test!(
        app,
        Method::GET,
        "/api/positions",
        "",
        StatusCode::OK,
        format!(
            r#"[{{"id":{},"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","time":"#,
            id1, user_id
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

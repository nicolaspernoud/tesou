#[macro_export]
macro_rules! do_test {
    ($app:expr, $method:expr, $uri:expr, $payload:expr, $expected_status_code:expr, $body_expect_to_start_with:expr) => {{
        use tower::ServiceExt;
        let serialized: String;
        let ptype = $crate::tester::type_of($payload);
        println!("Payload type: {}", ptype);
        if ptype == "&str" || ptype == "&alloc::string::String" {
            serialized = $payload.to_string();
        } else {
            serialized = serde_json::to_string($payload).expect("Could not serialized payload");
        }
        println!("Payload: {}", $payload);
        let req = http::Request::builder()
            .method($method)
            .uri($uri)
            .header("content-type", "application/json")
            .header("Authorization", "Bearer 0101")
            .body(axum::body::Body::from(serialized))
            .unwrap();
        let resp = $app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), $expected_status_code);
        let body_bytes = http_body_util::BodyExt::collect(resp.into_body()).await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body_bytes).unwrap().to_string();
        println!("Body: {}", body);
        assert!(body.starts_with(&$body_expect_to_start_with.to_string()));
        body
    }};
}

#[macro_export]
macro_rules! do_test_extract_id {
    ($app:expr, $method:expr, $uri:expr, $payload:expr, $expected_status_code:expr, $body_expect_to_start_with:expr) => {{
        use regex::Regex;
        let body = do_test!(
            $app,
            $method,
            $uri,
            $payload,
            $expected_status_code,
            $body_expect_to_start_with
        );
        let rg = Regex::new(r#""id":(\d+)"#).unwrap();
        match rg.captures(&body) {
            Some(x) => x.get(1).unwrap().as_str().parse().unwrap(),
            None => 0,
        }
    }};
}

pub fn type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

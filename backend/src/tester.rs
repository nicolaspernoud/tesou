#[macro_export]
macro_rules! do_test {
    ($app:expr, $method:expr, $uri:expr, $payload:expr, $expected_status_code:expr, $body_expect_to_start_with:expr) => {{
        let serialized: String;
        let ptype = $crate::tester::type_of($payload);
        println!("Payload type: {}", ptype);
        if ptype == "&str" || ptype == "&alloc::string::String" {
            serialized = $payload.to_string();
        } else {
            serialized = serde_json::to_string($payload).expect("Could not serialized payload");
        }
        println!("Payload: {}", $payload);
        let req = test::TestRequest::with_uri($uri)
            .method($method)
            .set_payload(serialized)
            .insert_header(("content-type", "application/json"))
            .insert_header(("Authorization", "Bearer 0101"))
            .to_request();
        let resp = test::call_service(&mut $app, req).await;
        assert_eq!(resp.status(), $expected_status_code);
        let body = test::read_body(resp).await;
        let body = std::str::from_utf8(&body).unwrap().to_string();
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

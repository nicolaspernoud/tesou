use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn random_string() -> std::string::String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

const HEALTH_CHECK_RESPONSE: &str = "all is well";

#[get("/hello/<name>")]
fn greet(name: &str) -> String {
    format!("Hello {}!", name)
}

#[get("/health-check")]
fn health_check() -> &'static str {
    HEALTH_CHECK_RESPONSE
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![greet, health_check])
}

#[cfg(test)]
mod test {
    use super::{rocket, HEALTH_CHECK_RESPONSE};
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn health_check_ok() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/health-check").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), HEALTH_CHECK_RESPONSE);
    }

    #[test]
    fn greet_ok() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/hello/mark").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello mark!");

        let response = client.get("/hello/Jason").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello Jason!");

        let response = client.get("/hello/234235").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello 234235!");
    }
}

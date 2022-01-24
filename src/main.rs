#[macro_use]
extern crate rocket;

use rocket::form::{Form, FromForm};
use rocket::{Build, Rocket};

const HEALTH_CHECK_RESPONSE: &str = "all is well";

#[derive(FromForm)]
pub struct NameEmailForm<'a> {
    name: &'a str,
    email: &'a str,
}

#[get("/hello/<name>")]
fn greet(name: &str) -> String {
    format!("Hello {}!", name)
}

#[get("/health-check")]
fn health_check() -> &'static str {
    HEALTH_CHECK_RESPONSE
}

#[post("/subscribe", data = "<form>")]
fn subscribe(form: Form<NameEmailForm>) -> String {
    format!("{}, {}", form.name, form.email)
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![greet, health_check, subscribe])
}

#[cfg(test)]
mod test {
    use super::{rocket, HEALTH_CHECK_RESPONSE};
    use rocket::http::{ContentType, Status};
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

    #[test]
    fn valid_subscription() {
        let body = "name=le%20guin&email=ursula.leguin%40gmail.com";
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .post("/subscribe")
            .header(ContentType::Form)
            .body(body)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().unwrap(),
            "le guin, ursula.leguin@gmail.com"
        );
    }

    // TODO test failure cases with data guards.
    #[test]
    fn invalid_subscriptions() {
        let invalid_forms = vec![
            "name=le%20guin",                  // missing email
            "email=ursula.leguin%40gmail.com", // missing name
            "",                                // missing both
        ];
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        for invalid in &invalid_forms {
            let response = client
                .post("/subscribe")
                .header(ContentType::Form)
                .body(invalid)
                .dispatch();
            assert_eq!(response.status(), Status::UnprocessableEntity);
        }
    }
}

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

mod schema;
use schema::subscriptions;

use diesel::RunQueryDsl;
use rocket::form::{Form, FromForm};
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

use std::ops::Deref;

type Result<T> = std::result::Result<T, Debug<diesel::result::Error>>;

#[database("newsletter_db")]
pub struct NewsletterDbConn(diesel::PgConnection);

const HEALTH_CHECK_RESPONSE: &str = "all is well";

#[derive(FromForm, Insertable)]
#[table_name = "subscriptions"]
pub struct NameEmailForm {
    name: String,
    email: String,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Subscription {
    id: uuid::Uuid,
    email: String,
    name: String,
    subscribed_at: chrono::DateTime<chrono::offset::Utc>,
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
async fn subscribe(
    db_conn: NewsletterDbConn,
    form: Form<NameEmailForm>,
) -> Result<Created<Json<Subscription>>> {
    let result: Result<Subscription> = db_conn
        .run(move |conn| {
            diesel::insert_into(subscriptions::table)
                .values(form.deref())
                .get_result(conn)
        })
        .await?;

    result.map(|value| {
        Created::new("/").body(Json(value))
    })
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![greet, health_check, subscribe])
        .attach(NewsletterDbConn::fairing())
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

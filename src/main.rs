#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

mod schema;
use schema::subscriptions;

use diesel::RunQueryDsl;
use rocket::form::{Form, FromForm};
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

type Result<T> = std::result::Result<T, Debug<diesel::result::Error>>;
type Timestamp = chrono::DateTime<chrono::offset::Utc>;

#[database("newsletter_db")]
pub struct NewsletterDbConn(diesel::PgConnection);

const HEALTH_CHECK_RESPONSE: &str = "all is well";

#[derive(FromForm, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct NameEmailForm {
    pub name: String,
    pub email: String,
}

#[derive(Insertable, Queryable, Serialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
#[table_name = "subscriptions"]
pub struct Subscription {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub subscribed_at: Timestamp,
}

impl From<NameEmailForm> for Subscription {
    fn from(form: NameEmailForm) -> Self {
        let uuid = uuid::Uuid::new_v4();
        let utc = chrono::offset::Utc::now();
        Self {
            id: uuid,
            email: form.email,
            name: form.name,
            subscribed_at: utc,
        }
    }
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
) -> Result<Created<Json<NameEmailForm>>> {
    let form_value = form.clone(); // TODO why the double clone is needed?
    db_conn
        .run(move |conn| {
            diesel::insert_into(subscriptions::table)
                .values(Subscription::from(form_value))
                .execute(conn)
        })
        .await?;

    Ok(Created::new("/").body(Json(form.clone()))) // TODO why the double clone
}

#[delete("/")]
async fn destroy(db_conn: NewsletterDbConn) -> Result<()> {
    db_conn
        .run(move |conn| diesel::delete(subscriptions::table).execute(conn))
        .await?;
    Ok(())
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![greet, health_check, subscribe, destroy])
        .attach(NewsletterDbConn::fairing())
}

#[cfg(test)]
mod test {
    use super::{rocket, HEALTH_CHECK_RESPONSE, NameEmailForm};
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
        assert_eq!(response.status(), Status::Created);
        let subscribed: NameEmailForm =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert_eq!(subscribed.name, "le guin");
        assert_eq!(subscribed.email, "ursula.leguin@gmail.com");
        client.delete("/").dispatch(); // remove inserted entry for reproducibility
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

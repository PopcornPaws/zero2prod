#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

mod schema;
use schema::subscriptions;

use diesel::{ExpressionMethods, RunQueryDsl, QueryDsl};
use rocket::form::{Form, FromForm};
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rocket_contrib::uuid::Uuid as RocketUuid;
use rocket_sync_db_pools::database;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Debug<diesel::result::Error>>;

#[database("newsletter_db")]
pub struct NewsletterDbConn(diesel::PgConnection);

const HEALTH_CHECK_RESPONSE: &str = "all is well";

#[derive(FromForm)]
pub struct NameEmailForm {
    name: String,
    email: String,
}

#[derive(Insertable, Queryable, Serialize, Clone)]
#[table_name = "subscriptions"]
#[serde(crate = "rocket::serde")]
pub struct Subscription {
    id: Uuid,
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

#[get("/")]
async fn list(
    db_conn: NewsletterDbConn,
) -> Result<Json<Vec<Uuid>>> {
    let ids: Vec<Uuid> = db_conn.run( move |conn| {
        subscriptions::table
            .select(subscriptions::id)
            .load(conn)
    }).await?;

    Ok(Json(ids))
}

#[post("/subscribe", data = "<form>")]
async fn subscribe(
    db_conn: NewsletterDbConn,
    form: Form<NameEmailForm>,
) -> Result<Created<Json<[String; 2]>>> {
    let subscription = Subscription {
        id: Uuid::new_v4(),
        email: form.email.clone(),
        name: form.name.clone(),
        subscribed_at: chrono::offset::Utc::now(),
    };
    db_conn
        .run(move |conn| {
            diesel::insert_into(subscriptions::table)
                .values(&subscription)
                .execute(conn)
        })
        .await?;

    Ok(Created::new("/").body(Json([form.name.clone(), form.email.clone()])))
}

#[delete("/<id>")]
async fn delete(
    db_conn: NewsletterDbConn,
    id: RocketUuid,
) -> Result<Option<()>> {
    let affected = db_conn.run(move |conn| {
        diesel::delete(subscriptions::table)
            .filter(subscriptions::id.eq(id.into_inner()))
            .execute(conn)
    }).await?;
    Ok((affected == 1).then(|| ()))
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
        // TODO delete posts after creating a new subscription
        // so that the test can be run again
        //let ids = client.get("/").dispatch();
        //println!("{:?}", ids);
        assert_eq!(response.status(), Status::Created);
        assert_eq!(
            response.into_string().unwrap(),
            "[\"le guin\", \"ursula.leguin@gmail.com\"]"
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

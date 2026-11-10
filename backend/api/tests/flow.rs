//! API tests against a real PostgreSQL. Needs TEST_DATABASE_URL, e.g.
//! postgres://postgres:geslo@localhost:5432/postgres. Every test gets its own new database.
use api::{router, AppState};
use axum_test::TestServer;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

async fn setup() -> Option<(TestServer, DatabaseConnection)> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL ni nastavljen, test preskočen");
        return None;
    };
    let admin = Database::connect(&url).await.unwrap();
    let name = format!("popravi_test_{}", uuid::Uuid::new_v4().simple());
    admin.execute_unprepared(&format!("CREATE DATABASE {name}")).await.unwrap();
    let base = url.rsplit_once('/').unwrap().0;
    let db = Database::connect(format!("{base}/{name}")).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    service::seed::ensure_admin(&db, "admin@popravi.test", "admingeslo").await.unwrap();
    service::seed::seed_demo(&db, "demogeslo").await.unwrap();
    let app = router(
        AppState {
            db: db.clone(),
            jwt_secret: "test-secret".into(),
        },
        CorsLayer::new(),
    );
    Some((TestServer::new(app).unwrap(), db))
}

async fn login(s: &TestServer, email: &str, pw: &str) -> String {
    let r = s.post("/api/v1/auth/login").json(&json!({"email": email, "password": pw})).await;
    r.assert_status_ok();
    format!("Bearer {}", r.json::<Value>()["token"].as_str().unwrap())
}

async fn category(s: &TestServer, slug: &str) -> String {
    let cats: Value = s.get("/api/v1/categories").await.json();
    cats.as_array().unwrap().iter().find(|c| c["slug"] == slug).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn in_days(n: i64) -> String {
    (service::today() + chrono::Duration::days(n)).to_string()
}

fn tomorrow() -> String {
    in_days(1)
}

async fn post_job(s: &TestServer, token: &str, cat: &str) -> Value {
    let r = s
        .post("/api/v1/job-requests")
        .add_header("Authorization", token)
        .json(&json!({
            "category_id": cat, "title": "Pušča kuhinjska pipa", "description": "Pipa v kuhinji pušča že tri dni, kaplja tudi ponoči.",
            "city": "Maribor", "postal_code": "2000", "address_line": "Ulica 1", "budget_min": 40, "budget_max": 80
        }))
        .await;
    r.assert_status(axum::http::StatusCode::CREATED);
    r.json()
}

#[tokio::test]
async fn us01_register_and_login() {
    let Some((s, db)) = setup().await else { return };
    let r = s
        .post("/api/v1/auth/register")
        .json(&json!({"email": "Nova@Primer.si", "full_name": "Nova Stranka", "password": "geslo1234", "role": "CUSTOMER"}))
        .await;
    r.assert_status(axum::http::StatusCode::CREATED);
    let r = s
        .post("/api/v1/auth/register")
        .json(&json!({"email": "nova@primer.si", "full_name": "X", "password": "geslo1234", "role": "CUSTOMER"}))
        .await;
    r.assert_status(axum::http::StatusCode::CONFLICT);
    assert_eq!(r.json::<Value>()["message"], "Račun s tem e-naslovom že obstaja.");
    let r = s
        .post("/api/v1/auth/register")
        .json(&json!({"email": "a@primer.si", "full_name": "X", "password": "geslo1234", "role": "ADMIN"}))
        .await;
    r.assert_status_bad_request();
    let r = s
        .post("/api/v1/auth/register")
        .json(&json!({"email": "b@primer.si", "full_name": "X", "password": "kratko", "role": "CUSTOMER"}))
        .await;
    r.assert_status_bad_request();
    assert_eq!(r.json::<Value>()["field"], "password");

    let row = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT password_hash FROM users WHERE email = 'nova@primer.si'",
        ))
        .await
        .unwrap()
        .unwrap();
    let hash: String = row.try_get("", "password_hash").unwrap();
    assert!(hash.starts_with("$argon2id$"));

    let r = s
        .post("/api/v1/auth/login")
        .json(&json!({"email": "nova@primer.si", "password": "napacno123"}))
        .await;
    r.assert_status_unauthorized();
    assert_eq!(r.json::<Value>()["message"], "Napačen e-naslov ali geslo.");
    let token = login(&s, "nova@primer.si", "geslo1234").await;
    s.get("/api/v1/auth/me")
        .add_header("Authorization", &token)
        .await
        .assert_status_ok();
    s.get("/api/v1/auth/me").await.assert_status_unauthorized();

    db.execute_unprepared("UPDATE users SET is_active = false WHERE email = 'nova@primer.si'")
        .await
        .unwrap();
    s.get("/api/v1/auth/me")
        .add_header("Authorization", &token)
        .await
        .assert_status_unauthorized();
    let r = s
        .post("/api/v1/auth/login")
        .json(&json!({"email": "nova@primer.si", "password": "geslo1234"}))
        .await;
    assert_ne!(r.status_code(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn us02_to_us06_full_flow() {
    let Some((s, db)) = setup().await else { return };
    let customer = login(&s, "stranka@popravi.test", "demogeslo").await;
    let provider = login(&s, "mojster@popravi.test", "demogeslo").await;
    let pending = login(&s, "novi.mojster@popravi.test", "demogeslo").await;
    let cat = category(&s, "vodovodna-dela").await;

    // US-02
    let bad = s
        .post("/api/v1/job-requests")
        .add_header("Authorization", &customer)
        .json(&json!({"category_id": cat, "title": "Pušča pipa", "description": "Pipa v kuhinji pušča že tri dni.", "city": "Maribor", "postal_code": "2000", "budget_min": 90, "budget_max": 50}))
        .await;
    bad.assert_status_bad_request();
    assert_eq!(bad.json::<Value>()["field"], "budget_max");
    s.post("/api/v1/job-requests")
        .add_header("Authorization", &provider)
        .json(&json!({}))
        .await
        .assert_status_unprocessable_entity();
    let job = post_job(&s, &customer, &cat).await;
    let id = job["id"].as_str().unwrap().to_string();
    assert_eq!(job["status"], "OPEN");
    assert_eq!(job["address_line"], "Ulica 1");
    let hist: Value = s
        .get(&format!("/api/v1/job-requests/{id}/history"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(hist[0]["from_status"], Value::Null);
    assert_eq!(hist[0]["to_status"], "OPEN");
    let days: Value = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            format!("SELECT (expires_at - created_at = interval '14 days') AS ok FROM job_requests WHERE id = '{id}'"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<bool>("", "ok")
        .unwrap()
        .into();
    assert_eq!(days, true);

    // US-03
    let m: Value = s
        .get("/api/v1/job-requests/matching")
        .add_header("Authorization", &provider)
        .await
        .json();
    assert_eq!(m["total"], 1);
    assert_eq!(m["items"][0]["quote_sent"], false);
    assert_eq!(m["verification_status"], "VERIFIED");
    let other_cat = category(&s, "pleskanje").await;
    post_job(&s, &customer, &other_cat).await;
    let m: Value = s
        .get("/api/v1/job-requests/matching")
        .add_header("Authorization", &provider)
        .await
        .json();
    assert_eq!(m["total"], 1, "druga kategorija se ne sme pokazati");
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &provider)
        .await
        .json();
    assert_eq!(d["address_line"], Value::Null, "ulica je skrita pred sprejemom");

    // US-04
    let q = json!({"price": 60, "estimated_hours": 1, "earliest_start": tomorrow(), "message": "Pridem v torek."});
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &pending)
        .json(&q)
        .await;
    r.assert_status_forbidden();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &provider)
        .json(&json!({"price": 0, "estimated_hours": 1, "earliest_start": tomorrow()}))
        .await;
    r.assert_status_bad_request();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &provider)
        .json(&q)
        .await;
    r.assert_status(axum::http::StatusCode::CREATED);
    let quote_id = r.json::<Value>()["id"].as_str().unwrap().to_string();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &provider)
        .json(&q)
        .await;
    r.assert_status(axum::http::StatusCode::CONFLICT);
    assert_eq!(r.json::<Value>()["message"], "Na to povpraševanje ste že oddali ponudbo.");
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(d["status"], "QUOTED");
    let r = s
        .put(&format!("/api/v1/quotes/{quote_id}"))
        .add_header("Authorization", &provider)
        .json(&json!({"price": 55.5, "estimated_hours": 1.5, "earliest_start": tomorrow()}))
        .await;
    r.assert_status_ok();
    assert_eq!(r.json::<Value>()["price"], 55.5);
    let m: Value = s
        .get("/api/v1/job-requests/matching")
        .add_header("Authorization", &provider)
        .await
        .json();
    assert_eq!(m["items"][0]["quote_sent"], true);
    assert_eq!(m["items"][0]["quote_count"], 1);

    // second provider so we can check REJECTED on accept
    db.execute_unprepared("UPDATE provider_profiles SET verification_status = 'VERIFIED'")
        .await
        .unwrap();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &pending)
        .json(&json!({"price": 90, "estimated_hours": 2, "earliest_start": tomorrow()}))
        .await;
    r.assert_status(axum::http::StatusCode::CREATED);
    let other_quote = r.json::<Value>()["id"].as_str().unwrap().to_string();

    // US-05
    s.get(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &provider)
        .await
        .assert_status_forbidden();
    let qs: Value = s
        .get(&format!("/api/v1/job-requests/{id}/quotes?sort=price"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(qs.as_array().unwrap().len(), 2);
    assert_eq!(qs[0]["price"], 55.5);
    s.post(&format!("/api/v1/quotes/{quote_id}/accept"))
        .add_header("Authorization", &provider)
        .await
        .assert_status_forbidden();
    s.post(&format!("/api/v1/quotes/{quote_id}/accept"))
        .add_header("Authorization", &customer)
        .await
        .assert_status_ok();
    let qs: Value = s
        .get(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &customer)
        .await
        .json();
    let status_of = |qid: &str| qs.as_array().unwrap().iter().find(|q| q["id"] == qid).unwrap()["status"].clone();
    assert_eq!(status_of(&quote_id), "ACCEPTED");
    assert_eq!(status_of(&other_quote), "REJECTED");
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(d["status"], "ACCEPTED");
    assert_eq!(d["accepted_quote_id"], quote_id.as_str());
    s.post(&format!("/api/v1/quotes/{other_quote}/accept"))
        .add_header("Authorization", &customer)
        .await
        .assert_status(axum::http::StatusCode::CONFLICT);
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &provider)
        .await
        .json();
    assert_eq!(d["address_line"], "Ulica 1");
    assert_eq!(d["customer"]["phone"], "040 111 222");
    s.put(&format!("/api/v1/quotes/{quote_id}"))
        .add_header("Authorization", &provider)
        .json(&q)
        .await
        .assert_status(axum::http::StatusCode::CONFLICT);

    // US-06
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &provider)
        .json(&json!({"status": "COMPLETED"}))
        .await;
    r.assert_status(axum::http::StatusCode::CONFLICT);
    assert_eq!(r.json::<Value>()["message"], "Ta sprememba statusa ni dovoljena.");
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &pending)
        .json(&json!({"status": "IN_PROGRESS"}))
        .await
        .assert_status_forbidden();
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &customer)
        .json(&json!({"status": "IN_PROGRESS"}))
        .await
        .assert_status_forbidden();
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &provider)
        .json(&json!({"status": "IN_PROGRESS"}))
        .await
        .assert_status_ok();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &customer)
        .json(&json!({"status": "CANCELLED", "note": "Ne potrebujem več."}))
        .await;
    assert_eq!(r.status_code(), axum::http::StatusCode::FORBIDDEN);
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &provider)
        .json(&json!({"status": "COMPLETED"}))
        .await
        .assert_status_ok();
    let hist: Value = s
        .get(&format!("/api/v1/job-requests/{id}/history"))
        .add_header("Authorization", &customer)
        .await
        .json();
    let steps: Vec<&str> = hist.as_array().unwrap().iter().map(|h| h["to_status"].as_str().unwrap()).collect();
    assert_eq!(steps, ["OPEN", "QUOTED", "ACCEPTED", "IN_PROGRESS", "COMPLETED"]);
    assert_eq!(hist[3]["from_status"], "ACCEPTED");
    assert_eq!(hist[3]["changed_by_name"], "Marko Kovač");
}

#[tokio::test]
async fn us06_customer_cancel_needs_reason() {
    let Some((s, _db)) = setup().await else { return };
    let customer = login(&s, "stranka@popravi.test", "demogeslo").await;
    let other = {
        s.post("/api/v1/auth/register")
            .json(&json!({"email": "druga@primer.si", "full_name": "Druga", "password": "geslo1234", "role": "CUSTOMER"}))
            .await;
        login(&s, "druga@primer.si", "geslo1234").await
    };
    let cat = category(&s, "elektrika").await;
    let id = post_job(&s, &customer, &cat).await["id"].as_str().unwrap().to_string();
    s.get(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &other)
        .await
        .assert_status_forbidden();
    s.get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &other)
        .await
        .assert_status_forbidden();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &customer)
        .json(&json!({"status": "CANCELLED", "note": "kratko"}))
        .await;
    r.assert_status_bad_request();
    assert_eq!(r.json::<Value>()["field"], "note");
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &other)
        .json(&json!({"status": "CANCELLED", "note": "Tuje povpraševanje"}))
        .await
        .assert_status_forbidden();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &customer)
        .json(&json!({"status": "CANCELLED", "note": "Sem že našla mojstra."}))
        .await;
    r.assert_status_ok();
    assert_eq!(r.json::<Value>()["status"], "CANCELLED");
    s.post(&format!("/api/v1/job-requests/{id}/status"))
        .add_header("Authorization", &customer)
        .json(&json!({"status": "OPEN"}))
        .await
        .assert_status(axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn us04_withdraw_quote() {
    let Some((s, _db)) = setup().await else { return };
    let customer = login(&s, "stranka@popravi.test", "demogeslo").await;
    let provider = login(&s, "mojster@popravi.test", "demogeslo").await;
    let cat = category(&s, "vodovodna-dela").await;
    let id = post_job(&s, &customer, &cat).await["id"].as_str().unwrap().to_string();
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &provider)
        .json(&json!({"price": 70, "estimated_hours": 2, "earliest_start": tomorrow()}))
        .await;
    let qid = r.json::<Value>()["id"].as_str().unwrap().to_string();
    let r = s
        .post(&format!("/api/v1/quotes/{qid}/withdraw"))
        .add_header("Authorization", &provider)
        .await;
    r.assert_status_ok();
    assert_eq!(r.json::<Value>()["status"], "WITHDRAWN");
    let mine: Value = s.get("/api/v1/quotes/mine").add_header("Authorization", &provider).await.json();
    assert_eq!(mine[0]["quote"]["status"], "WITHDRAWN");
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(d["status"], "QUOTED", "workflow nima prehoda nazaj v OPEN");
    assert_eq!(d["active_quote_count"], 0);
    let qs: Value = s
        .get(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(qs.as_array().unwrap().len(), 0);
    s.post(&format!("/api/v1/quotes/{qid}/withdraw"))
        .add_header("Authorization", &provider)
        .await
        .assert_status(axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn us05_sort_and_us04_date_round_trip() {
    let Some((s, db)) = setup().await else { return };
    db.execute_unprepared("UPDATE provider_profiles SET verification_status = 'VERIFIED'")
        .await
        .unwrap();
    let customer = login(&s, "stranka@popravi.test", "demogeslo").await;
    let a = login(&s, "mojster@popravi.test", "demogeslo").await;
    let b = login(&s, "novi.mojster@popravi.test", "demogeslo").await;
    let cat = category(&s, "vodovodna-dela").await;
    let id = post_job(&s, &customer, &cat).await["id"].as_str().unwrap().to_string();
    let late = in_days(5);
    let r = s
        .post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &a)
        .json(&json!({"price": 50, "estimated_hours": 1, "earliest_start": late}))
        .await;
    assert_eq!(r.json::<Value>()["earliest_start"], late.as_str());
    s.post(&format!("/api/v1/job-requests/{id}/quotes"))
        .add_header("Authorization", &b)
        .json(&json!({"price": 90, "estimated_hours": 1, "earliest_start": tomorrow()}))
        .await
        .assert_status(axum::http::StatusCode::CREATED);
    let by_price: Value = s
        .get(&format!("/api/v1/job-requests/{id}/quotes?sort=price"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(by_price[0]["price"], 50.0);
    assert_eq!(by_price[1]["price"], 90.0);
    let by_start: Value = s
        .get(&format!("/api/v1/job-requests/{id}/quotes?sort=start"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(by_start[0]["earliest_start"], tomorrow().as_str());
    assert_eq!(by_start[1]["earliest_start"], late.as_str());
    let d: Value = s
        .get(&format!("/api/v1/job-requests/{id}"))
        .add_header("Authorization", &customer)
        .await
        .json();
    assert_eq!(d["active_quote_count"], 2);
}

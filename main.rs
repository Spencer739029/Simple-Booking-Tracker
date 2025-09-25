use axum::{
    extract::{Form, Path},
    routing::{get, post},
    response::{Redirect, Html},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use tokio::fs;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Submission {
    name: String,
    address: String,
    #[serde(default)]
    booked_on: String,
    #[serde(default)]
    booking_date: String,
    #[serde(default)]
    booking_time: String,
    #[serde(default, deserialize_with = "checkbox_bool", serialize_with = "bool_to_checkbox")]
    completed: bool,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/submissions", get(show_submissions))
        .route("/submit", post(submit_successful))
        .route("/delete/:index", post(delete_submission))
        .route("/toggle_completed/:id", post(toggle_completed))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}

async fn delete_submission(Path(index): Path<usize>) -> Redirect {
    let path = "submissions.json";
    let data = fs::read_to_string(path).await.unwrap_or_else(|_| "[]".to_string());
    let mut submissions: Vec<Submission> = serde_json::from_str(&data).unwrap_or_else(|_| Vec::new());

    if index < submissions.len() {
        submissions.remove(index);
        if let Ok(json) = serde_json::to_string_pretty(&submissions) {
            let _ = fs::write(path, json).await;
        }
    }

    Redirect::to("/submissions")
}

fn is_duplicate_time(new_date: &str, new_time: &str, submissions: &[Submission]) -> bool {
    if let Some(new_parsed) = parse_time(new_time) {
        submissions.iter().any(|s| {
            if s.booking_date != new_date { return false; }
            parse_time(&s.booking_time)
                .map(|t| (new_parsed - t).num_minutes().abs() < 45)
                .unwrap_or(false)
        })
    } else {
        false
    }
}

fn checkbox_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.map(|s| s == "on").unwrap_or(false))
}

fn bool_to_checkbox<S>(b: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if *b { serializer.serialize_str("on") } else { serializer.serialize_str("") }
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/form.html"))
}

fn parse_time(time_str: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M").ok()
}

async fn submit_successful(Form(mut form): Form<Submission>) -> Html<String> {
    form.booked_on = Local::now().format("%Y-%m-%d").to_string();
    let path = "submissions.json";
    let mut submissions: Vec<Submission> = match fs::read_to_string(path).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };

    if is_duplicate_time(&form.booking_date, &form.booking_time, &submissions) {
        return Html("<h1>Time already booked!</h1><a href='/'>Go back</a>".to_string());
    }

    submissions.push(form.clone());
    if let Ok(json) = serde_json::to_string_pretty(&submissions) {
        let _ = fs::write(path, json).await;
    }

    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Submission Successful</title>
        <style>
            body {{
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background-color: #f0f2f5;
                display: flex;
                justify-content: center;
                align-items: center;
                height: 100vh;
                margin: 0;
            }}
            .card {{
                background: #ffffff;
                border-radius: 12px;
                padding: 2rem 3rem;
                box-shadow: 0 10px 25px rgba(0,0,0,0.1);
                max-width: 450px;
                width: 90%;
                text-align: center;
            }}
            .card h1 {{
                color: #28a745;
                margin-bottom: 1.5rem;
                font-size: 1.8rem;
            }}
            .card p {{
                font-size: 1rem;
                margin: 0.6rem 0;
                color: #333;
            }}
            .card .back-btn {{
                display: inline-block;
                margin-top: 1.8rem;
                padding: 10px 20px;
                background-color: #007bff;
                color: #fff;
                border: none;
                border-radius: 6px;
                text-decoration: none;
                font-weight: bold;
                transition: background 0.3s;
            }}
            .card .back-btn:hover {{
                background-color: #0056b3;
            }}
        </style>
    </head>
    <body>
        <div class="card">
            <h1>Booking Successful!</h1>
            <p><strong>Name:</strong> {}</p>
            <p><strong>Address:</strong> {}</p>
            <p><strong>Date:</strong> {}</p>
            <p><strong>Time:</strong> {}</p>
            <p><strong>Completed:</strong> {}</p>
            <a class="back-btn" href="/submissions">View All Bookings</a>
        </div>
    </body>
    </html>
    "#,
    form.name, form.address, form.booking_date, form.booking_time, form.completed);

    Html(html)
}

async fn show_submissions() -> Html<String> {
    let path = "submissions.json";
    let data = fs::read_to_string(path).await.unwrap_or_else(|_| "[]".to_string());
    let submissions: Vec<Submission> = serde_json::from_str(&data).unwrap_or_else(|_| Vec::new());

    let mut html = String::from(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Bookings</title>
        <style>
            body { font-family: Arial, sans-serif; background-color: #f4f4f9; padding: 2rem; text-align: center; }
            table { border-collapse: collapse; width: 90%; margin: 0 auto; background: white; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
            th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
            th { background-color: #007bff; color: white; }
            tr:nth-child(even) { background-color: #f9f9f9; }
            form { display: inline; }
            input[type="checkbox"] { width: 20px; height: 20px; }
            button { background-color: #007bff; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; transition: all 0.3s ease; }
            button:hover { background-color: #0056b3; }
            a.back-link { display: inline-block; margin-top: 20px; text-decoration: none; font-weight: bold; color: #007bff; }
            a.back-link:hover { text-decoration: underline; }
        </style>
    </head>
    <body>
        <h1>Bookings</h1>
        <table>
            <tr>
                <th>Name</th>
                <th>Address</th>
                <th>Booked On</th>
                <th>Date</th>
                <th>Completed</th>
                <th>Action</th>
            </tr>
    "#);

    for (i, s) in submissions.iter().enumerate() {
        html.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>
                    <form action='/toggle_completed/{}' method='post'>
                        <input type='checkbox' name='completed' onchange='this.form.submit()' {} />
                    </form>
                </td>
                <td>
                    <form action='/delete/{}' method='post'>
                        <button type='submit'>Delete</button>
                    </form>
                </td>
            </tr>"#,
            s.name, s.address, s.booked_on, s.booking_date,
            i, if s.completed { "checked" } else { "" }, i
        ));
    }

    html.push_str(r#"</table>
        <a class="back-link" href="/">Go back</a>
    </body>
    </html>"#);

    Html(html)
}

async fn toggle_completed(Path(id): Path<usize>, Form(data): Form<HashMap<String, String>>) -> Html<String> {
    let path = "submissions.json";
    let mut submissions: Vec<Submission> = match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    if let Some(sub) = submissions.get_mut(id) {
        sub.completed = data.get("completed").is_some();
    }

    if let Ok(json) = serde_json::to_string_pretty(&submissions) {
        let _ = fs::write(path, json).await;
    }

    Html("<meta http-equiv='refresh' content='0; url=/submissions' />".to_string())
}

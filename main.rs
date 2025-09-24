use axum::{
    extract::Form,
    routing::{get, post},
    response::Html,
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Submission {
    name: String,
    address: String,
    #[serde(default)]
    booked_on: String,  // will be set in Rust after submission
    #[serde(default)]
    booking_date: String,
    #[serde(default)]
    booking_time: String,
}

#[tokio::main]
async fn main() {
    // build app with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/submissions", get(show_submissions))
        .route("/submit", post(submit_successful))
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

fn is_duplicate_time(new_time: &str, submissions: &[Submission]) -> bool {
    if let Some(new_parsed) = parse_time(new_time) {
        submissions.iter().any(|s| {
            parse_time(&s.booking_time)
                .map(|t| t == new_parsed)
                .unwrap_or(false)
        })
    } else {
        false
    }
}


// serve the form page
async fn index() -> Html<&'static str> {
    Html(include_str!("../static/form.html"))
}

fn parse_time(time_str: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M").ok()
}

async fn show_submissions() -> Html<String> {
    let path = "submissions.json";

    let data = fs::read_to_string(path)
        .await
        .unwrap_or_else(|_| "[]".to_string());

    let submissions: Vec<Submission> = serde_json::from_str(&data).unwrap_or_else(|_| Vec::new());

    let mut html = String::from(
        "<h1>Bookings</h1>\
         <table border='1'>\
         <tr><th>Name</th><th>Address</th><th>Booked On</th><th>Date</th></tr>"
    );

    for s in submissions {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            s.name, s.address, s.booked_on, s.booking_date
        ));
    }

    html.push_str("</table><a href='/'>Go back</a>");

    Html(html)
}

async fn submit_successful(Form(mut form): Form<Submission>) -> Html<String> {
    // Set booked_on date
    form.booked_on = Local::now().format("%Y-%m-%d").to_string();

    // Load existing submissions
    let path = "submissions.json";
    let mut submissions: Vec<Submission> = match fs::read_to_string(path).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };

    if is_duplicate_time(&form.booking_time, &submissions) {
        return Html(format!(
            "<h1>Time already booked!</h1><a href='/'>Go back</a>"
        ));
    }

    // Save the new submission
    submissions.push(form.clone());
    if let Ok(json) = serde_json::to_string_pretty(&submissions) {
        let _ = fs::write(path, json).await;
    }

    // Generate styled confirmation HTML
    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Submission Successful</title>
        <style>
            body {{
                font-family: Arial, sans-serif;
                background-color: #f4f4f9;
                padding: 2rem;
                text-align: center;
                min-height: 100vh;
            }}
            h1 {{
                color: #333;
                margin-bottom: 1rem;
            }}
            .confirmation {{
                background-color: white;
                border: 2px solid #007bff;
                padding: 2rem;
                max-width: 500px;
                margin: 0 auto;
                border-radius: 8px;
                box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            }}
            .confirmation p {{
                font-size: 1.1rem;
                margin-bottom: 0.8rem;
            }}
            .confirmation a {{
                display: inline-block;
                margin-top: 1.5rem;
                text-decoration: none;
                font-weight: bold;
                color: #007bff;
                padding: 8px 16px;
                border: 1px solid #007bff;
                border-radius: 4px;
                transition: all 0.3s ease;
            }}
            .confirmation a:hover {{
                background-color: #007bff;
                color: white;
            }}
        </style>
    </head>
    <body>
        <div class="confirmation">
            <h1>Submission Successful!</h1>
            <p><strong>Name:</strong> {}</p>
            <p><strong>Address:</strong> {}</p>
            <p><strong>Date:</strong> {}</p>
            <p><strong>Time:</strong> {}</p>
            <a href="/">Go back</a>
        </div>
    </body>
    </html>
    "#,
    form.name, form.address, form.booking_date, form.booking_time);

    Html(html)
}

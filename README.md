This is a Rust and Axum-based web application for tracking bookings. Key features include submitting, viewing, completing, and deleting bookings, with data persisted in a JSON file. It prevents double-booking within 45 minutes of the same date.

**Features:**

*   Booking submissions with name, address, date, time, and completion status.
*   Double-booking prevention (45-minute window).
*   Styled table view of all bookings.
*   Completion status via checkbox.
*   Individual booking deletion.
*   Styled submission confirmation page.

**Technologies:**

*   Rust (core language)
*   Axum (web framework)
*   Tokio (async runtime)
*   Serde (JSON serialization)
*   Chrono (date/time)
*   Tower HTTP (static files)
*   HTML/CSS (frontend)

**Project Structure:**

*   `Cargo.toml`
*   `src/main.rs` (application logic/routing)
*   `submissions.json` (booking data)
*   `static/` (CSS, JS)
*   `form.html` (booking form)

**Getting Started:**

Prerequisites: Rust, Cargo

Installation:

1.  `git clone <repository_url>`
2.  `cd booking_tracker`
3.  `cargo build`
4.  `cargo run`
5.  Access at `http://127.0.0.1:3000`

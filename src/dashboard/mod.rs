use rocket::response::content::RawHtml;

#[get("/")]
pub fn render() -> RawHtml<&'static str> {
    RawHtml(include_str!("dashboard.html"))
}

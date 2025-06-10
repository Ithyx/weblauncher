use rocket::{http::ContentType, response::content::RawHtml};

#[get("/")]
pub fn render() -> RawHtml<&'static str> {
    RawHtml(include_str!("dashboard.html"))
}

#[get("/favicon.ico")]
pub fn favicon() -> (ContentType, &'static [u8]) {
    let bytes = include_bytes!("wl_favico.png");

    (ContentType::Icon, bytes)
}

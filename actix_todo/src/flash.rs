use actix_web::error::Result;
use actix_web::middleware::session::RequestSession;
use actix_web::HttpRequest;

pub fn set_flash<T>(request: &HttpRequest<T>, flash: FlashMessage) {
    request
        .session()
        .set("flash", flash)
        .expect("failed to set cookie");
}

pub fn get_flash<T>(req: &HttpRequest<T>) -> Result<Option<FlashMessage>> {
    req.session().get::<FlashMessage>("flash")
}

#[derive(Deserialize, Serialize)]
pub struct FlashMessage {
    pub kind: String,
    pub message: String,
}

impl FlashMessage {
    pub fn success(message: &str) -> Self {
        Self {
            kind: "success".to_owned(),
            message: message.to_owned(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            kind: "error".to_owned(),
            message: message.to_owned(),
        }
    }
}

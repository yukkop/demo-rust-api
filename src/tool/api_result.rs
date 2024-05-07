use rocket::Request;
use rocket::response::{self, Response};
use rocket::response::Responder;
use rocket::http::{ContentType, Status};

pub struct ApiError(pub String, pub Status);

impl ApiError {
    #[allow(dead_code)]
    pub fn condlict_str(message: &str) -> Self {
        Self(message.into(), Status::Conflict)
    }

    #[allow(dead_code)]
    pub fn conflict(message: String) -> Self {
        Self(message, Status::Conflict)
    }

    #[allow(dead_code)]
    pub fn bad_request_str(message: &str) -> Self {
        Self(message.into(), Status::BadRequest)
    }

    #[allow(dead_code)]
    pub fn bad_request(message: String) -> Self {
        Self(message, Status::BadRequest)
    }
}

impl<'r, 'o> Responder<'r, 'o> for ApiError
where
    'o: 'r,
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let json_error = serde_json::json!({ "error": self.0 });

        Response::build()
            .sized_body(json_error.to_string().len(), std::io::Cursor::new(json_error.to_string()))
            .header(ContentType::JSON)
            .status(self.1)
            .ok()
    }
}
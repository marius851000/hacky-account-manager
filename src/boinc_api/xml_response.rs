use actix_web::{error::ErrorInternalServerError, HttpResponse, Result};
use serde::Serialize;

pub fn xml_to_response<T: Serialize>(value: T, root: &str) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body(
        quick_xml::se::to_string_with_root(root, &value)
            .map_err(|_| ErrorInternalServerError("Encoding the XML"))?,
    ))
}

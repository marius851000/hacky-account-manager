use crate::boinc_api::xml_to_response;
use actix_web::{get, HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
struct GetProjectResult {
    name: String,
    min_passwd_length: u32,
    account_manager: Option<()>,
    uses_username: Option<()>,
    client_acount_creation_disabled: Option<()>,
}

#[get("/get_project_config.php")]
pub async fn get_project_config() -> Result<HttpResponse> {
    //TODO: at least check the password against the list of valid password
    Ok(xml_to_response(
        GetProjectResult {
            name: "Test Project Manager".to_string(),
            min_passwd_length: 1,
            account_manager: Some(()),
            uses_username: Some(()),
            client_acount_creation_disabled: Some(()),
        },
        "project_config",
    )?)
}

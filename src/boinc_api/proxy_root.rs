use actix_web::{
    get,
    web::{self, Data},
    HttpResponse,
};

use crate::AppState;

#[get("/proxy/{project_id}/")]
pub async fn proxy_root_route(
    project: web::Path<String>,
    app_state: Data<AppState>,
) -> HttpResponse {
    let scheduler_url = app_state.get_scheduler_url(&project);
    HttpResponse::Ok().body(format!(
        r#"<!DOCTYPE html>
<html>
    <head>
        <meta>
            <!--<scheduler>{}</scheduler>-->
            <link rel="boinc_scheduler" href="{}">
        </meta>
    </head>
</html>"#,
        scheduler_url, scheduler_url
    ))
}

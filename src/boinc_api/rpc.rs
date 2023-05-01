use crate::{boinc_api::xml_to_response, planify_action, AppState, DeviceInfo, PlanificatorResult, device_info::HostInfo};
use actix_web::{post, web::Data, HttpResponse, Result};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct RpcAccount {
    url: String,
    url_signature: String,
    authenticator: String,
    resource_share: u16,
    detach: u8
}

#[derive(Serialize)]
pub struct RpcResponse {
    name: String,
    signing_key: String,
    account: Vec<RpcAccount>,
}

impl RpcResponse {
    pub fn new_from_planificator_result(
        app_state: &AppState,
        plan_result: &PlanificatorResult,
    ) -> Result<Self> {
        let mut account = Vec::new();
        for (project_id, project) in &app_state.projects {
            let mut priority = 0;
            if let Some(project_plan) = plan_result.projects.get(project_id) {
                priority = project_plan.priority;
            }
            account.push(RpcAccount {
                url: app_state.get_proxy_url(&project_id),
                url_signature: project.url_signature.clone(),
                authenticator: project.authenticator.clone(),
                resource_share: priority,
                detach: if priority == 0 { 1 } else { 0 },
            });
        }
        Ok(RpcResponse {
            name: app_state.account_manager_name.clone(),
            signing_key: app_state.signing_key.clone(),
            account,
        })
    }
}

#[derive(Deserialize)]
pub struct RpcQuery {
    name: String,
    host_info: HostInfo
}


#[post("/rpc.php")]
pub async fn rpc_endpoint(post: String, app_state: Data<AppState>) -> Result<HttpResponse> {
    let rpc_query: RpcQuery = quick_xml::de::from_str(&post).unwrap();
    if rpc_query.name != app_state.weak_auth {
        return Ok(HttpResponse::Forbidden().body("Invalid name"));
    }
    let plan_result = planify_action(&app_state, &DeviceInfo {
        host_info: rpc_query.host_info
    });
    let result = RpcResponse::new_from_planificator_result(&app_state, &plan_result)?;

    xml_to_response(result, "acct_mgr_reply")
}

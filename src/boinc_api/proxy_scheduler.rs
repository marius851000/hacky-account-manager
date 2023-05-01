use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{
    http::StatusCode,
    post,
    web::{self, Data},
    HttpRequest, HttpResponse, HttpResponseBuilder,
};
use awc::Client;
use serde::Deserialize;

use crate::{database::WorkUnit, device_info::HostInfo, AppState, AppVersion};

#[derive(Deserialize, Debug)]
pub struct SchedulerWorkUnit {
    /// The name of task (there is a also a unique id, but it isn’t communicated this way)
    name: String,
    app_name: String,
    /// estimated computation in FLOPs
    #[serde(default)]
    rsc_fpops_est: f64,
    /// max permitted FLOPs
    #[serde(default)]
    rsc_fpops_bound: f64,
    /// max permitted memory usage
    #[serde(default)]
    rsc_memory_bound: f64,
    /// max disk usage
    #[serde(default)]
    rsc_disk_bound: f64,
}

#[derive(Deserialize, Debug)]
pub struct SchedulerResult {
    wu_name: String,
    name: String,
    platform: String,
    version_num: u64,
    plan_class: String,
}

#[derive(Deserialize, Debug)]
pub struct SchedulerApp {
    name: String,
    user_friendly_name: String,
}

#[derive(Deserialize, Debug)]
pub struct SchedulerAppVersion {
    app_name: String,
    version_num: u64,
    platform: String,
    #[serde(default)]
    plan_class: String,
}

#[derive(Deserialize, Debug)]
pub struct SchedulerReply {
    #[serde(default)]
    workunit: Vec<SchedulerWorkUnit>,
    #[serde(default)]
    result: Vec<SchedulerResult>,
    #[serde(default)]
    app: Vec<SchedulerApp>,
    #[serde(default)]
    app_version: Vec<SchedulerAppVersion>,
}

#[derive(Deserialize, Debug)]
pub struct ResultQuery {
    name: String,
    state: u64,
}

#[derive(Deserialize, Debug)]
pub struct Query {
    #[serde(default)]
    result: Vec<ResultQuery>,
    host_info: HostInfo,
}

#[post("/proxy/{project_id}/scheduler")]
pub async fn proxy_scheduler_route(
    request: HttpRequest,
    source_body: String,
    path: web::Path<String>,
    app_state: Data<AppState>,
) -> HttpResponse {
    //TODO: convert all error to boinc-readable error (except 404)
    //info!("source:\n{}", source_body);
    let project_id = path.into_inner();
    let project = if let Some(v) = app_state.projects.get(&project_id) {
        v
    } else {
        return HttpResponse::NotFound().body("Project not found");
    };

    let user_agent = if let Some(ua) = request.headers().get("User-Agent") {
        ua
    } else {
        return HttpResponse::BadRequest().body("No user-agent provided");
    };

    let query_analyzed: Query = quick_xml::de::from_str(&source_body).unwrap();
    for result in &query_analyzed.result {
        app_state
            .database
            .update_status(&project_id, &result.name, result.state)
            .unwrap();
    }
    //debug!("{:?}", query_analyzed);

    //TODO: get rid of unwrap
    let mut res = Client::default()
        .post(&project.scheduler_url)
        .insert_header(("User-Agent", user_agent))
        .send_body(source_body)
        .await
        .unwrap();

    //TODO: avoid unwrap
    let result_body = res.body().await.unwrap();

    //info!("result\n{:?}", result_body);

    if res.status() == StatusCode::OK {
        let result_string = String::from_utf8_lossy(&result_body).replace("&", "&amp;"); // The server doesn’t seems to escape it, which is invalid XML!!
        let result: SchedulerReply = quick_xml::de::from_str(&result_string).unwrap();

        for workunit in &result.workunit {
            for res in &result.result {
                if res.wu_name == workunit.name {
                    let merged_wu = WorkUnit {
                        cpid: query_analyzed.host_info.host_cpid.clone(),
                        project: project_id.clone(),
                        name: workunit.name.to_string(),
                        status: 1,
                        app_name: workunit.app_name.to_string(),
                        rsc_fpops_est: workunit.rsc_fpops_est,
                        rsc_fpops_bound: workunit.rsc_fpops_bound,
                        rsc_memory_bound: workunit.rsc_memory_bound,
                        rsc_disk_bound: workunit.rsc_disk_bound,
                        platform: res.platform.to_string(),
                        version_num: res.version_num,
                        plan_class: res.plan_class.to_string(),
                        result_name: res.name.to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    app_state.database.add_work_unit(&merged_wu).unwrap();
                    break;
                }
            }
        }

        for app_version in &result.app_version {
            for app in &result.app {
                if app.name == app_version.app_name {
                    let merged_app = AppVersion {
                        project: project_id.clone(),
                        app_name: app.name.clone(),
                        user_friendly_name: app.user_friendly_name.clone(),
                        version: app_version.version_num,
                        platform: app_version.platform.clone(),
                        plan_class: app_version.plan_class.clone(),
                    };
                    app_state.database.add_app_version(&merged_app).unwrap();
                    break;
                };
            }
        }
        //println!("{:?}", result);
    }

    HttpResponseBuilder::new(res.status()).body(result_body)
}

use std::{collections::HashMap};

use crate::{AppState, DeviceInfo};

pub struct PlanificatorProject {
    pub priority: u16,
}

pub struct PlanificatorResult {
    pub projects: HashMap<String, PlanificatorProject>,
}

impl PlanificatorResult {
    pub fn new_from_app_state(app_state: &AppState, default_priority: u16) -> Self {
        let mut projects = HashMap::new();
        for (project_id, _) in &app_state.projects {
            projects.insert(
                project_id.to_string(),
                PlanificatorProject {
                    priority: default_priority,
                },
            );
        }
        PlanificatorResult { projects }
    }
}

pub fn planify_action(app_state: &AppState, device_info: &DeviceInfo) -> PlanificatorResult {
    // step 1: get the list of project, add it to result
    let mut tasks = PlanificatorResult::new_from_app_state(app_state, 100);
    
    // step 2: if PC os string contains NixOS, remove LODA
    if device_info.host_info.os_name.to_lowercase().contains("nixos") {
        tasks.projects.remove("loda");
    }

    // step 3: collect the amount of fpop task received for this device recently
    let workunits = app_state.database.list_workunit_sent_since(&device_info.host_info.host_cpid, 0).unwrap();
    let mut sent_wu = HashMap::new();
    // TODO: automatically discard those with too much failure
    for wu in &workunits {
        if wu.status == 6 { // cancelled
            continue;
        }
        sent_wu.entry(wu.project.to_string()).and_modify(|x| *x += wu.rsc_fpops_est).or_insert(wu.rsc_fpops_est);
    }
    sent_wu.entry("worldcommunitygrid".to_string()).and_modify(|x| *x /= 2.0);

    for (project_id, project) in tasks.projects.iter_mut() {
        if let Some(swu) = sent_wu.get(project_id) {
            let tflop = swu / 1_000_000_000_000.0;
            let prio_change = (10_000f64 / tflop.max(10_000f64)) * 1_000f64;
            println!("{}: {}, {}", project_id, prio_change, tflop);
            project.priority += prio_change as u16;
        } else {
            project.priority += 1000;
        }
    }

    // step 3: that’s it for now. Later add the improved ressource sharing algo from science united
    return tasks;
}

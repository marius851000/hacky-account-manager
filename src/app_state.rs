use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Context;
use log::info;
use serde::Deserialize;

use crate::DataBase;

#[derive(Clone)]
pub struct AppState {
    pub projects: HashMap<String, Project>,
    pub account_manager_name: String,
    pub signing_key: String,
    pub base_url: String,
    pub database: DataBase,
    pub weak_auth: String,
}

#[derive(Clone)]
pub struct Project {
    pub name: String,
    pub scheduler_url: String,
    pub url_signature: String,
    pub authenticator: String,
}

#[derive(Deserialize)]
struct JsonProject {
    name: String,
    scheduler_url: String,
    authenticator: String,
}

#[derive(Deserialize)]
struct JsonConfig {
    projects: HashMap<String, JsonProject>,
    account_manager_name: String,
    signing_key_path: String,
    signature_folder: String,
    base_url: String,
    weak_auth: String,
}

impl AppState {
    pub fn new<T: Read>(reader: &mut T, database: DataBase) -> anyhow::Result<Self> {
        let config: JsonConfig =
            serde_json::from_reader(reader).context("Failed to read the config.json file")?;
        let mut signing_key = String::new();
        File::open(&config.signing_key_path)
            .unwrap()
            .read_to_string(&mut signing_key)
            .unwrap();

        let mut projects = HashMap::new();
        for (project_key, project_data) in &config.projects {
            let project_proxy_url = Self::_get_proxy_url(&config.base_url, project_key);
            info!("looking for the signature of “{}”", project_proxy_url);

            let mut url_signature = String::new();
            let path = Path::new(&config.signature_folder);
            File::open(&path.join(format!("{}.pub", project_key)))
                .unwrap()
                .read_to_string(&mut url_signature)
                .unwrap();

            projects.insert(
                project_key.clone(),
                Project {
                    name: project_data.name.clone(),
                    scheduler_url: project_data.scheduler_url.clone(),
                    authenticator: project_data.authenticator.clone(),
                    url_signature,
                },
            );
        }

        let result = AppState {
            projects,
            account_manager_name: config.account_manager_name,
            signing_key: signing_key,
            base_url: config.base_url,
            database,
            weak_auth: config.weak_auth,
        };
        Ok(result)
    }

    pub fn get_proxy_url(&self, project: &str) -> String {
        Self::_get_proxy_url(&self.base_url, project)
    }

    fn _get_proxy_url(base_url: &str, project: &str) -> String {
        format!("{}/proxy/{}/", base_url, project)
    }

    pub fn get_scheduler_url(&self, project: &str) -> String {
        format!("{}/proxy/{}/scheduler", self.base_url, project)
    }
}

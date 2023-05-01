use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use rusqlite::Connection;

pub struct AppVersion {
    pub project: String,
    pub app_name: String,
    pub user_friendly_name: String,
    pub version: u64,
    pub platform: String,
    pub plan_class: String,
}

#[derive(Debug)]
pub struct WorkUnit {
    pub cpid: String,
    pub project: String,
    pub name: String,
    pub status: u32,
    pub app_name: String,
    pub rsc_fpops_est: f64,
    pub rsc_fpops_bound: f64,
    pub rsc_memory_bound: f64,
    pub rsc_disk_bound: f64,
    pub platform: String,
    pub version_num: u64,
    pub plan_class: String,
    pub result_name: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct DataBase {
    conn: Arc<Mutex<Connection>>,
}

impl DataBase {
    pub fn new(db_path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path).context("Opening the sqlite database")?;
        //TODO: properly detect the state of the database
        Self::upgrade(&conn).context("upgrading/seeding the database")?;
        Ok(DataBase {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Security: table_name should be a trusted input
    fn check_table_exist(conn: &Connection, table_name: &str) -> anyhow::Result<bool> {
        Ok(conn
            .prepare(&format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                table_name
            ))?
            .query([])?
            .next()?
            .is_some())
    }

    fn upgrade(conn: &Connection) -> anyhow::Result<()> {
        if !Self::check_table_exist(conn, "workunit")? {
            conn.execute(
                "CREATE TABLE workunit (
                    cpid TEXT,
                    result_name TEXT,
                    name TEXT,
                    project TEXT,
                    status NUMBER,
                    app_name TEXT,
                    rsc_fpops_est NUMBER,
                    rsc_fpops_bound NUMBER,
                    rsc_memory_bound NUMBER,
                    rsc_disk_bound NUMBER,
                    platform TEXT,
                    version_num NUMBER,
                    plan_class TEXT,
                    timestamp NUMBER,
                    PRIMARY KEY(result_name, project)
                )",
                (),
            )
            .context("Creating the workunit table")?;
        };
        if !Self::check_table_exist(conn, "app_version")? {
            conn.execute(
                "CREATE TABLE app_version (
                    project TEXT,
                    app_name TEXT,
                    user_friendly_name TEXT,
                    version NUMBER,
                    platform TEXT,
                    plan_class TEXT,
                    PRIMARY KEY(project, app_name, version, platform, plan_class)
                )",
                (),
            )
            .context("Creating the app_version table")?;
        }

        Ok(())
    }

    pub fn add_app_version(&self, app_version: &AppVersion) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.prepare_cached("INSERT OR IGNORE INTO app_version VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .unwrap()
            .execute((
                &app_version.project,
                &app_version.app_name,
                &app_version.user_friendly_name,
                app_version.version,
                &app_version.platform,
                &app_version.plan_class,
            ))?;
        Ok(())
    }

    pub fn add_work_unit(&self, workunit: &WorkUnit) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        //TODO: somehow save the device...
        conn.prepare_cached("INSERT OR IGNORE INTO workunit VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)")
            .unwrap()
            .execute((
                &workunit.cpid,
                &workunit.result_name,
                &workunit.name,
                &workunit.project,
                workunit.status,
                &workunit.app_name,
                workunit.rsc_fpops_est,
                workunit.rsc_fpops_bound,
                workunit.rsc_memory_bound,
                workunit.rsc_disk_bound,
                &workunit.platform,
                workunit.version_num,
                &workunit.plan_class,
                workunit.timestamp
            ))?;
        Ok(())
    }

    pub fn update_status(&self, project: &str, name: &str, status: u64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.prepare_cached("UPDATE workunit SET status=?1 WHERE project=?2 AND result_name=?3")
            .unwrap()
            .execute((status, project, name))?;
        Ok(())
    }

    pub fn list_workunit_sent_since(
        &self,
        cpid: &str,
        timestampt: u64,
    ) -> anyhow::Result<Vec<WorkUnit>> {
        Ok(self.conn.lock().unwrap().prepare_cached("SELECT cpid, result_name, name, project, status, app_name, rsc_fpops_est, rsc_fpops_bound, rsc_memory_bound, rsc_disk_bound, platform, version_num, plan_class, timestamp FROM workunit WHERE cpid=?1 AND timestamp > ?2").unwrap().query_map((&cpid, timestampt), |row| {
            Ok(WorkUnit {
                cpid: row.get(0)?,
                result_name: row.get(1)?,
                name: row.get(2)?,
                project: row.get(3)?,
                status: row.get(4)?,
                app_name: row.get(5)?,
                rsc_fpops_est: row.get(6)?,
                rsc_fpops_bound: row.get(7)?,
                rsc_memory_bound: row.get(8)?,
                rsc_disk_bound: row.get(9)?,
                platform: row.get(10)?,
                version_num: row.get(11)?,
                plan_class: row.get(12)?,
                timestamp: row.get(13)?
            })
        }).unwrap().map(|x| x.unwrap()).collect())
    }
}

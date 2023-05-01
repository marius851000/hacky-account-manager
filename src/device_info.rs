use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HostInfo {
    pub os_name: String,
    pub os_version: String,
    pub host_cpid: String,
}

pub struct DeviceInfo {
    pub host_info: HostInfo,
}

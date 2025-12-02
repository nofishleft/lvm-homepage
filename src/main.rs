use std::process::Command;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::http::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct PhysicalVolume {
    name: String,
    vg_name: String,
    size_gb: f64,
    free_gb: f64,
    used_percent: f64,
}

fn get_physical_volumes() -> Result<Vec<PhysicalVolume>, String> {
    let output = Command::new("pvs")
        .args(&[
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "pv_name,vg_name,pv_size,pv_free",
        ])
        .output()
        .map_err(|e| format!("Failed to execute pvs: {}", e))?;

    if !output.status.success() {
        return Err(format!("pvs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = str::from_utf8(&output.stdout).map_err(|e| format!("Failed to parse pvs output: {}", e))?;

    let pvs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                Some(PhysicalVolume {
                    name: parts[0].to_string(),
                    vg_name: parts[1].to_string(),
                    size_gb: parts[2].parse::<f64>().unwrap_or(f64::NAN),
                    free_gb: parts[3].parse::<f64>().unwrap_or(f64::NAN),
                    used_percent: (parts[2].parse::<f64>().unwrap_or(f64::NAN) - parts[3].parse::<f64>().unwrap_or(f64::NAN)) / parts[2].parse::<f64>().unwrap_or(f64::NAN) * 100.0,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(pvs)
}

#[derive(Debug, Serialize, Deserialize)]
struct VolumeGroup {
    name: String,
    pv_count: u32,
    lv_count: u32,
    size_gb: f64,
    free_gb: f64,
    used_percent: f64,
}

fn get_volume_groups() -> Result<Vec<VolumeGroup>, String> {
    let output = Command::new("vgs")
        .args(&[
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "vg_name,pv_count,lv_count,vg_size,vg_free",
        ])
        .output()
        .map_err(|e| format!("Failed to execute vgs: {}", e))?;

    if !output.status.success() {
        return Err(format!("vgs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Failed to parse vgs output: {}", e))?;

    let vgs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                Some(VolumeGroup {
                    name: parts[0].to_string(),
                    pv_count: parts[1].parse::<u32>().unwrap_or(u32::MAX),
                    lv_count: parts[2].parse::<u32>().unwrap_or(u32::MAX),
                    size_gb: parts[3].parse::<f64>().unwrap_or(f64::NAN),
                    free_gb: parts[4].parse::<f64>().unwrap_or(f64::NAN),
                    used_percent: (parts[3].parse::<f64>().unwrap_or(f64::NAN) - parts[4].parse::<f64>().unwrap_or(f64::NAN)) / parts[3].parse::<f64>().unwrap_or(f64::NAN) * 100.0,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(vgs)
}

#[derive(Debug, Serialize, Deserialize)]
struct LogicalVolume {
    name: String,
    vg_name: String,
    size_gb: f64,
    path: String,
    active: bool,
}

fn get_logical_volumes() -> Result<Vec<LogicalVolume>, String> {
    let output = Command::new("lvs")
        .args(&[
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "lv_name,lv_size,lv_attr,vg_name",
        ])
        .output()
        .map_err(|e| format!("Failed to execute lvs: {}", e))?;

    if !output.status.success() {
        return Err(format!("lvs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Failed to parse lvs output: {}", e))?;

    let lvs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                Some(LogicalVolume {
                    name: parts[0].to_string(),
                    vg_name: parts[1].to_string(),
                    size_gb: parts[2].parse::<f64>().unwrap_or(f64::NAN),
                    path: parts[3].to_string(),
                    active: parts[4] == "active",
                })
            } else {
                None
            }
        })
        .collect();

    Ok(lvs)
}

#[derive(Debug, Serialize, Deserialize)]
struct LvmStatus {
    physical_volumes: Vec<PhysicalVolume>,
    volume_groups: Vec<VolumeGroup>,
    logical_volumes: Vec<LogicalVolume>,
}

async fn lvm_status() -> Result<HttpResponse, Error> {
    match (
        get_physical_volumes(),
        get_volume_groups(),
        get_logical_volumes(),
    ) {
        (Ok(pvs), Ok(vgs), Ok(lvs)) => {
            let status = LvmStatus {
                physical_volumes: pvs,
                volume_groups: vgs,
                logical_volumes: lvs,
            };
            Ok(HttpResponse::Ok().json(status))
        }
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/lvm", web::get().to(lvm_status))
    })
        .bind("127.0.0.1:9000")?
        .run()
        .await
}

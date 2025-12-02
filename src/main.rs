use actix_web::http::Error;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct PhysicalVolume {
    name: String,
    value: String,
}

fn get_physical_volumes() -> Result<Vec<PhysicalVolume>, String> {
    let output = Command::new("sudo")
        .args(&[
            "pvs",//command
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "pv_name,vg_name,pv_size,pv_missing",
        ])
        .output()
        .map_err(|e| format!("Failed to execute pvs: {}", e))?;

    if !output.status.success() {
        return Err(format!("pvs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = std::str::from_utf8(&output.stdout).map_err(|e| format!("Failed to parse pvs output: {}", e))?;

    let pvs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let health = {
                    if parts[3].is_empty() {
                        "ok"
                    } else {
                        "failed"
                    }
                };
                let size = {
                    let size = parts[2].parse::<f64>().unwrap_or(0.0) as u32;
                    if size == 0 {
                        "".to_string()
                    } else {
                        size.to_string()
                    }
                };
                Some(PhysicalVolume {
                    name: parts[0].to_string(),
                    value: format!("{}, {}g, {}", parts[1], size, health),
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
    value: String,
}

fn get_volume_groups() -> Result<Vec<VolumeGroup>, String> {
    let output = Command::new("sudo")
        .args(&[
            "vgs",//command
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "vg_name,pv_count,vg_missing_pv_count,vg_size",
        ])
        .output()
        .map_err(|e| format!("Failed to execute vgs: {}", e))?;

    if !output.status.success() {
        return Err(format!("vgs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|e| format!("Failed to parse vgs output: {}", e))?;

    let vgs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let health = {
                    if parts[2] == "0" {
                        "ok".to_string()
                    } else {
                        format!("failed({}/{})", parts[2], parts[1])
                    }
                };
                let size = {
                    let size = parts[3].parse::<f64>().unwrap_or(0.0) as u32;
                    if size == 0 {
                        "".to_string()
                    } else {
                        size.to_string()
                    }
                };
                Some(VolumeGroup {
                    name: parts[0].to_string(),
                    value: format!("{}g, {}", size, health),
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
    value: String,
}

fn get_logical_volumes() -> Result<Vec<LogicalVolume>, String> {
    let output = Command::new("sudo")
        .args(&[
            "lvs",//command
            "--noheadings",
            "--units",
            "g",
            "--nosuffix",
            "--separator",
            ",",
            "--options",
            "lv_full_name,lv_size,lv_active,lv_health_status",
        ])
        .output()
        .map_err(|e| format!("Failed to execute lvs: {}", e))?;

    if !output.status.success() {
        return Err(format!("lvs command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|e| format!("Failed to parse lvs output: {}", e))?;

    let lvs = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let mut health = parts[3];
                if health.is_empty() {
                    health = "ok";
                }
                let size = {
                    let size = parts[1].parse::<f64>().unwrap_or(0.0) as u32;
                    if size == 0 {
                        "".to_string()
                    } else {
                        size.to_string()
                    }
                };
                Some(LogicalVolume {
                    name: parts[0].to_string(),
                    value: format!("{}g, {}, {}", size, parts[2], health),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(lvs)
}

async fn lvs_route() -> Result<HttpResponse, Error> {
    match get_logical_volumes() {
        Ok(lvs) => {
            Ok(HttpResponse::Ok().json(lvs))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

async fn vgs_route() -> Result<HttpResponse, Error> {
    match get_volume_groups() {
        Ok(vgs) => {
            Ok(HttpResponse::Ok().json(vgs))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

async fn pvs_route() -> Result<HttpResponse, Error> {
    match get_physical_volumes() {
        Ok(pvs) => {
            Ok(HttpResponse::Ok().json(pvs))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());

    HttpServer::new(|| {
        App::new()
            .route("/lvs", web::get().to(lvs_route))
            .route("/vgs", web::get().to(vgs_route))
            .route("/pvs", web::get().to(pvs_route))
    })
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
}

use napi::{bindgen_prelude::*, Result};
use napi_derive::napi;
use brightness::blocking::{Brightness};
use serde_json::json;

#[napi]
pub fn list_devices() -> Result<String> {
    let devices_info = brightness::blocking::brightness_devices()
        .filter_map(|dev_result| dev_result.map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).ok())
        .map(|dev| {
            let raw_device_name = dev.device_name().map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).unwrap_or_else(|_| "Unknown Device".to_string());
            
            let normalized_name = raw_device_name.replace('\\', "/");
            let real_friendly_name = dev.display_name().map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).unwrap_or_else(|_| Some("Unknown Device".to_string()));
            let device_brightness = dev.get().map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).ok();

            json!({
                "device_name": normalized_name,
                "friendly_name": real_friendly_name,
                "current_brightness": device_brightness
            })
        })
        .collect::<Vec<_>>();
    let json_output = json!({
        "devices": devices_info
    });
    Ok(serde_json::to_string_pretty(&json_output).unwrap())
}

#[napi]
pub fn set_brightness(device_name: String, percentage: u32) -> Result<()> {
    let normalized_display_name = device_name.replace('\\', "/");
    let devices = brightness::blocking::brightness_devices()
        .filter_map(|dev_result| dev_result.map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).ok())
        .filter(|dev| {
            dev.device_name().map_or(false, |name| {
                let normalized_name = name.replace('\\', "/");
                normalized_name.contains(&normalized_display_name)
            })
        })
        .collect::<Vec<_>>();

    if devices.is_empty() {
        return Err(Error::new(Status::InvalidArg, "No matching display found."));
    }

    for dev in devices {
        dev.set(percentage).map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set brightness: {}", e)))?;
    }
    Ok(())
}

#[napi]
pub fn get_brightness(device_name: String) -> Result<u32> {
    let normalized_display_name = device_name.replace('\\', "/");
    let device = brightness::blocking::brightness_devices()
        .filter_map(|dev_result| dev_result.map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).ok())
        .find(|dev| {
            dev.device_name().map_or(false, |name| {
                let normalized_name = name.replace('\\', "/");
                normalized_name.contains(&normalized_display_name)
            })
        });

    match device {
        Some(dev) => Ok(dev.get().map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)))?),
        None => Err(Error::new(Status::InvalidArg, "No matching display found.")),
    }
}
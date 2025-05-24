use napi::{bindgen_prelude::*, Result};
use napi_derive::napi;
use brightness::blocking::{Brightness};
use serde_json::json;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};
use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;

fn get_real_display_name(device_name: &str) -> Option<String> {
    unsafe {
        let mut device_index = 0;
        let mut device = DISPLAY_DEVICEW::default();

        loop {
            let null_pcwstr = PCWSTR(std::ptr::null_mut());
            let result = EnumDisplayDevicesW(null_pcwstr, device_index, &mut device, 0);

            if !result.as_bool() {
                let err = std::io::Error::last_os_error().raw_os_error()?;
                if err == ERROR_NO_MORE_ITEMS.0 as i32 {
                    break None;
                } else {
                    break None;
                }
            }

            // Convert DeviceName and DeviceString from UTF-16
            let dev_name = String::from_utf16_lossy(&device.DeviceName);
            let dev_string = String::from_utf16_lossy(&device.DeviceString);

            // Match based on device name
            if dev_name.contains(device_name) || device_name.contains(&dev_name) {
                let friendly_name = dev_string.trim_matches(char::from(0)).to_string();
                break Some(friendly_name);
            }

            device_index += 1;
        }
    }
}

#[napi]
pub fn list_devices() -> Result<String> {
    let devices_info = brightness::blocking::brightness_devices()
        .filter_map(|dev_result| dev_result.map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).ok())
        .map(|dev| {
            let raw_device_name = dev.device_name().map_err(|e| Error::new(Status::GenericFailure, format!("{}", e))).unwrap_or_else(|_| "Unknown Device".to_string());
            let normalized_name = raw_device_name.replace('\\', "/");
            let real_friendly_name = get_real_display_name(&raw_device_name);

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
//! Screenshot capture functionality (macOS only).

/// Capture a screenshot of the app window.
///
/// On macOS, uses Core Graphics to find and capture the window.
/// On other platforms, returns an error.
#[cfg(not(tarpaulin_include))]
pub fn capture_screenshot(app_name: &str, output_path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let window_id = find_window_id(app_name)?;
        capture_window_to_png(window_id, output_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app_name, output_path);
        Err("Screenshot capture only supported on macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
#[cfg(not(tarpaulin_include))]
fn find_window_id(app_name: &str) -> Result<u32, String> {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionaryRef;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_graphics::window::{
        kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo,
    };

    let windows =
        unsafe { CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID) };

    if windows.is_null() {
        return Err("Failed to get window list".to_string());
    }

    let count = unsafe { core_foundation::array::CFArrayGetCount(windows) };
    let app_clean: String = app_name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    let mut candidates: Vec<(String, String, u32)> = Vec::new();

    for i in 0..count {
        let dict = unsafe {
            core_foundation::array::CFArrayGetValueAtIndex(windows, i) as CFDictionaryRef
        };

        if dict.is_null() {
            continue;
        }

        let owner_key = CFString::new("kCGWindowOwnerName");
        let owner_ptr = unsafe {
            core_foundation::dictionary::CFDictionaryGetValue(dict, owner_key.as_CFTypeRef() as _)
        };

        if owner_ptr.is_null() {
            continue;
        }

        let owner: CFString = unsafe {
            TCFType::wrap_under_get_rule(owner_ptr as core_foundation::string::CFStringRef)
        };
        let owner_str = owner.to_string();

        // Get window name
        let name_key = CFString::new("kCGWindowName");
        let name_ptr = unsafe {
            core_foundation::dictionary::CFDictionaryGetValue(dict, name_key.as_CFTypeRef() as _)
        };

        let name_str = if !name_ptr.is_null() {
            let name: CFString = unsafe {
                TCFType::wrap_under_get_rule(name_ptr as core_foundation::string::CFStringRef)
            };
            name.to_string()
        } else {
            String::new()
        };

        // Get window ID
        let id_key = CFString::new("kCGWindowNumber");
        let id_ptr = unsafe {
            core_foundation::dictionary::CFDictionaryGetValue(dict, id_key.as_CFTypeRef() as _)
        };

        if id_ptr.is_null() {
            continue;
        }

        let id_num: CFNumber =
            unsafe { TCFType::wrap_under_get_rule(id_ptr as core_foundation::number::CFNumberRef) };

        let window_id = match id_num.to_i32() {
            Some(id) => id as u32,
            None => continue,
        };

        // Skip windows with empty names (background windows, menus, etc.)
        if name_str.is_empty() {
            continue;
        }

        let owner_clean: String = owner_str
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        let name_clean: String = name_str
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        // Check if app name matches owner OR window name
        if owner_clean.contains(&app_clean)
            || app_clean.contains(&owner_clean)
            || name_clean.contains(&app_clean)
            || app_clean.contains(&name_clean)
        {
            return Ok(window_id);
        }

        // Track as candidate for error message
        candidates.push((owner_str, name_str, window_id));
    }

    // No match found - provide helpful error with available windows
    if candidates.is_empty() {
        Err(format!(
            "No windows found matching '{}'. No visible windows available.",
            app_name
        ))
    } else {
        let window_list: Vec<String> = candidates
            .iter()
            .take(5)
            .map(|(owner, name, _)| format!("'{}' ({})", owner, name))
            .collect();
        Err(format!(
            "No window found matching '{}'. Available: {}",
            app_name,
            window_list.join(", ")
        ))
    }
}

#[cfg(target_os = "macos")]
mod cg_ffi {
    use std::ffi::c_void;

    pub type CGImageRef = *const c_void;
    pub type CGDataProviderRef = *const c_void;
    pub type CFDataRef = *const c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGImageGetWidth(image: CGImageRef) -> usize;
        pub fn CGImageGetHeight(image: CGImageRef) -> usize;
        pub fn CGImageGetBytesPerRow(image: CGImageRef) -> usize;
        pub fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
        pub fn CGImageRelease(image: CGImageRef);
        pub fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFDataGetLength(data: CFDataRef) -> isize;
        pub fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
        pub fn CFRelease(cf: *const c_void);
    }
}

#[cfg(target_os = "macos")]
#[cfg(not(tarpaulin_include))]
fn capture_window_to_png(window_id: u32, output_path: &str) -> Result<(), String> {
    use cg_ffi::*;
    use core_graphics::display::CGRectNull;
    use core_graphics::window::{
        kCGWindowImageBoundsIgnoreFraming, kCGWindowImageDefault,
        kCGWindowListOptionIncludingWindow, CGWindowListCreateImage,
    };

    let wid = window_id;

    let image = unsafe {
        CGWindowListCreateImage(
            CGRectNull,
            kCGWindowListOptionIncludingWindow,
            wid,
            kCGWindowImageDefault | kCGWindowImageBoundsIgnoreFraming,
        )
    };

    if image.is_null() {
        return Err("Failed to capture window image".to_string());
    }

    let image = image as CGImageRef;
    let width = unsafe { CGImageGetWidth(image) };
    let height = unsafe { CGImageGetHeight(image) };
    let bytes_per_row = unsafe { CGImageGetBytesPerRow(image) };
    let data_provider = unsafe { CGImageGetDataProvider(image) };

    if data_provider.is_null() {
        unsafe { CGImageRelease(image) };
        return Err("Failed to get image data provider".to_string());
    }

    let cf_data = unsafe { CGDataProviderCopyData(data_provider) };
    if cf_data.is_null() {
        unsafe { CGImageRelease(image) };
        return Err("Failed to copy image data".to_string());
    }

    let length = unsafe { CFDataGetLength(cf_data) } as usize;
    let ptr = unsafe { CFDataGetBytePtr(cf_data) };
    let raw_data = unsafe { std::slice::from_raw_parts(ptr, length) };

    // Convert BGRA to RGBA
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let offset = y * bytes_per_row + x * 4;
            if offset + 3 < length {
                rgba_data.push(raw_data[offset + 2]); // R
                rgba_data.push(raw_data[offset + 1]); // G
                rgba_data.push(raw_data[offset]); // B
                rgba_data.push(raw_data[offset + 3]); // A
            }
        }
    }

    unsafe {
        CFRelease(cf_data as _);
        CGImageRelease(image);
    }

    let img: image::RgbaImage =
        image::ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
            .ok_or("Failed to create image buffer")?;

    img.save(output_path)
        .map_err(|e| format!("Failed to save PNG: {}", e))
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_capture_unsupported() {
        let result = super::capture_screenshot("test", "/tmp/test.png");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only supported on macOS"));
    }
}

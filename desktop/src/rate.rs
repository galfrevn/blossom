use std::ffi::{CStr, c_char};

use objc2::runtime::{AnyClass, AnyObject, Bool};
use objc2::{msg_send, sel};
use tauri::{Runtime, WebviewWindow};

const PREFERENCES_CLASS: &CStr = c"WKPreferences";
const SIXTY_FPS_FEATURE: &CStr = c"PreferPageRenderingUpdatesNear60FPSEnabled";

pub fn draw_at_the_display_rate<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.with_webview(|webview| lift_the_sixty_fps_cap(webview.inner().cast()))
}

#[allow(unsafe_code)]
fn lift_the_sixty_fps_cap(web_view: *mut AnyObject) {
    if web_view.is_null() {
        return;
    }
    let Some(preferences_class) = AnyClass::get(PREFERENCES_CLASS) else {
        return;
    };
    unsafe {
        let lists_its_features: Bool =
            msg_send![preferences_class, respondsToSelector: sel!(_features)];
        if !lists_its_features.as_bool() {
            return;
        }
        let configuration: *mut AnyObject = msg_send![web_view, configuration];
        if configuration.is_null() {
            return;
        }
        let preferences: *mut AnyObject = msg_send![configuration, preferences];
        if preferences.is_null() {
            return;
        }
        let switches_features: Bool =
            msg_send![preferences, respondsToSelector: sel!(_setEnabled:forFeature:)];
        if !switches_features.as_bool() {
            return;
        }
        let features: *mut AnyObject = msg_send![preferences_class, _features];
        if features.is_null() {
            return;
        }
        let feature_count: usize = msg_send![features, count];
        for index in 0..feature_count {
            let feature: *mut AnyObject = msg_send![features, objectAtIndex: index];
            let key: *mut AnyObject = msg_send![feature, key];
            if key.is_null() {
                continue;
            }
            let key_text: *const c_char = msg_send![key, UTF8String];
            if key_text.is_null() || CStr::from_ptr(key_text) != SIXTY_FPS_FEATURE {
                continue;
            }
            let _: () = msg_send![preferences, _setEnabled: Bool::NO, forFeature: feature];
            return;
        }
    }
}

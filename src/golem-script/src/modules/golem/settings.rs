use crate::HostData;
use boa_engine::object::builtins::JsDate;
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, js_string, Context, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use golem_ui::application::menu::style::MenuStyleFontSize;
use golem_ui::data::settings::DateTimeFormat;
use tracing::{debug, error, trace};

struct JsDatetimeFormat(DateTimeFormat);

impl Into<JsValue> for JsDatetimeFormat {
    fn into(self) -> JsValue {
        JsValue::String(match self.0 {
            DateTimeFormat::Default => js_string!("default"),
            DateTimeFormat::Short => js_string!("short"),
            DateTimeFormat::TimeOnly => js_string!("timeOnly"),
            DateTimeFormat::Hidden => js_string!("hidden"),
        })
    }
}

impl TryFromJs for JsDatetimeFormat {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value
            .to_string(context)?
            .to_std_string_lossy()
            .to_lowercase();

        Ok(Self(match s.as_str() {
            "default" => DateTimeFormat::Default,
            "short" => DateTimeFormat::Short,
            "timeonly" | "time" => DateTimeFormat::TimeOnly,
            "hidden" | "off" => DateTimeFormat::Hidden,
            _ => return Err(js_error!("Invalid datetime format")),
        }))
    }
}

#[derive(Debug, Clone, Copy)]
enum FontSize {
    Small,
    Medium,
    Large,
}

impl Into<MenuStyleFontSize> for FontSize {
    fn into(self) -> MenuStyleFontSize {
        match self {
            Self::Small => MenuStyleFontSize::Small,
            Self::Medium => MenuStyleFontSize::Medium,
            Self::Large => MenuStyleFontSize::Large,
        }
    }
}

impl Into<FontSize> for MenuStyleFontSize {
    fn into(self) -> FontSize {
        match self {
            MenuStyleFontSize::Small => FontSize::Small,
            MenuStyleFontSize::Medium => FontSize::Medium,
            MenuStyleFontSize::Large => FontSize::Large,
        }
    }
}

impl Into<JsValue> for FontSize {
    fn into(self) -> JsValue {
        JsValue::String(match self {
            Self::Small => js_string!("small"),
            Self::Medium => js_string!("medium"),
            Self::Large => js_string!("large"),
        })
    }
}

impl TryFromJs for FontSize {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value
            .to_string(context)?
            .to_std_string_lossy()
            .to_lowercase();

        match s.as_str() {
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            _ => Err(js_error!("Invalid font size")),
        }
    }
}

fn set_font_size_(ContextData(data): ContextData<HostData>, size: FontSize) {
    data.app_mut()
        .ui_settings_mut()
        .set_menu_font_size(size.into());
}

fn font_size_(ContextData(data): ContextData<HostData>) -> FontSize {
    data.app().ui_settings().menu_font_size().into()
}

fn set_datetime_format_(ContextData(data): ContextData<HostData>, datetime: JsDatetimeFormat) {
    data.app_mut()
        .ui_settings_mut()
        .set_toolbar_datetime_format(datetime.0);
}

fn datetime_format_(ContextData(data): ContextData<HostData>) -> JsDatetimeFormat {
    JsDatetimeFormat(data.app().ui_settings().toolbar_datetime_format())
}

fn set_show_fps_(ContextData(data): ContextData<HostData>, show: bool) {
    data.app_mut().ui_settings_mut().set_show_fps(show);
}

fn show_fps_(ContextData(data): ContextData<HostData>) -> bool {
    data.app().ui_settings().show_fps()
}

fn set_invert_toolbar_(ContextData(data): ContextData<HostData>, show: bool) {
    data.app_mut().ui_settings_mut().set_invert_toolbar(show);
}

fn invert_toolbar_(ContextData(data): ContextData<HostData>) -> bool {
    data.app().ui_settings().invert_toolbar()
}

fn list_time_zones_() -> Vec<JsString> {
    let root = "/usr/share/zoneinfo/posix/";

    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_string_lossy().to_string())
        .map(|path| {
            let path = path.trim_start_matches(root).trim_start_matches('/');
            js_string!(path)
        })
        .collect()
}

fn set_date_time_inner_(datetime: &str) -> JsResult<()> {
    // Make sure this is a valid date and time.
    let _ = time::OffsetDateTime::parse(
        datetime,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|e| js_error!("Invalid date and time: {}", e))?;

    let status = std::process::Command::new("date")
        .args(&["-s", datetime])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| js_error!("Could not set date and time: {}", e))?;

    if !status.success() {
        Err(js_error!("Could not set date and time"))
    } else {
        Ok(())
    }
}

fn update_date_time_(tz: Option<String>, update_tz: Option<bool>) {
    // Only deserialize the fields we care about.
    #[derive(Debug, serde::Deserialize)]
    struct WorldTimeApiResponse {
        timezone: String,
        datetime: String,
    }

    fn ping_ntp(tz: Option<String>) -> JsResult<WorldTimeApiResponse> {
        trace!(?tz, "Pinging worldtimeapi for time");

        let mut url = "https://worldtimeapi.org/api/ip".to_string();
        if let Some(tz) = tz {
            url = format!("https://worldtimeapi.org/api/timezone/{}", tz);
        }

        let result = reqwest::blocking::get(&url)
            .map_err(|e| js_error!("Could not get timezone from worldtimeapi: {}", e))?
            .json::<WorldTimeApiResponse>()
            .map_err(|e| js_error!("Could not parse worldtimeapi response: {}", e))?;
        trace!(?result, "Got time from worldtimeapi");
        Ok(result)
    }

    std::thread::spawn(move || {
        // Ignore errors.
        if let Some(dt) = ping_ntp(tz).ok() {
            if let Some(true) = update_tz {
                if let Err(e) = set_time_zone_(dt.timezone) {
                    error!(?e, "Could not set timezone");
                }
            }

            if let Err(e) = set_date_time_inner_(&dt.datetime) {
                error!(?e, "Could not set date and time");
            }
        }
    });
}

fn get_time_zone_() -> JsResult<Option<JsString>> {
    let prefix = "/usr/share/zoneinfo/posix/";
    let tz = std::fs::read_link("/etc/localtime")
        .map_err(|e| js_error!("Could not read /etc/localtime: {}", e))?;
    let tz = tz.to_string_lossy().to_string();
    if tz.starts_with(prefix) {
        Ok(Some(JsString::from(tz.trim_start_matches(prefix))))
    } else {
        Ok(None)
    }
}

fn set_time_zone_(tz: String) -> JsResult<()> {
    let tz_path = format!("/usr/share/zoneinfo/posix/{}", tz);

    // Check if the timezone exists in our system
    match std::fs::exists(&tz_path) {
        Ok(false) => return Err(js_error!("Timezone not found: {}", tz)),
        Err(e) => return Err(js_error!("Could not check timezone: {}", e)),
        Ok(true) => {
            debug!(tz, "Timezone found")
        }
    }

    std::fs::remove_file("/etc/localtime")
        .map_err(|e| js_error!("Could not remove /etc/localtime: {}", e))?;

    std::os::unix::fs::symlink(&tz_path, "/etc/localtime")
        .map_err(|e| js_error!("Could not create symlink: {}", e))?;

    Ok(())
}

fn set_date_time_(datetime: JsDate, context: &mut Context) -> JsResult<()> {
    let iso = datetime
        .to_iso_string(context)?
        .as_string()
        .ok_or_else(|| js_error!("Could not convert date to string"))?
        .to_std_string_lossy();

    set_date_time_inner_(&iso)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("settings"),
        [
            (
                js_string!("setFontSize"),
                set_font_size_.into_js_function_copied(context),
            ),
            (
                js_string!("fontSize"),
                font_size_.into_js_function_copied(context),
            ),
            (
                js_string!("setDatetimeFormat"),
                set_datetime_format_.into_js_function_copied(context),
            ),
            (
                js_string!("datetimeFormat"),
                datetime_format_.into_js_function_copied(context),
            ),
            (
                js_string!("setShowFps"),
                set_show_fps_.into_js_function_copied(context),
            ),
            (
                js_string!("showFps"),
                show_fps_.into_js_function_copied(context),
            ),
            (
                js_string!("setInvertToolbar"),
                set_invert_toolbar_.into_js_function_copied(context),
            ),
            (
                js_string!("invertToolbar"),
                invert_toolbar_.into_js_function_copied(context),
            ),
            (
                js_string!("updateDateTime"),
                update_date_time_.into_js_function_copied(context),
            ),
            (
                js_string!("listTimeZones"),
                list_time_zones_.into_js_function_copied(context),
            ),
            (
                js_string!("getTimeZone"),
                get_time_zone_.into_js_function_copied(context),
            ),
            (
                js_string!("setTimeZone"),
                set_time_zone_.into_js_function_copied(context),
            ),
            (
                js_string!("setDateTime"),
                set_date_time_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

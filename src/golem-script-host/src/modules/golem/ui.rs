use boa_engine::object::builtins::JsArray;
use boa_engine::{
    js_string, Context, Finalize, JsData, JsNativeError, JsResult, JsString, JsValue, Module, Trace,
};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::TryFromJs;

use golem_ui::application::menu;

use crate::HostData;

mod filesystem;

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuAction {
    Select(usize),
    Details(usize),
    Sort,
    Back,
}

impl menu::style::MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }

    fn into_details(self) -> Option<Self> {
        match self {
            MenuAction::Select(i) => Some(MenuAction::Details(i)),
            _ => None,
        }
    }

    fn sort() -> Option<Self> {
        Some(MenuAction::Sort)
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextMenuItem {
    label: String,
    marker: Option<String>,
    select: Option<JsValue>,
    details: Option<JsValue>,
    index: usize,
}

impl boa_engine::value::TryFromJs for TextMenuItem {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let object = match value {
            JsValue::Object(o) => o,
            JsValue::String(s) => {
                return Ok(Self {
                    label: s.to_std_string().unwrap(),
                    marker: None,
                    select: None,
                    details: None,
                    index: 0,
                });
            }
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("cannot convert value to a MenuItem")
                    .into());
            }
        };

        let label = object
            .get(js_string!("label"), context)?
            .to_string(context)?
            .to_std_string()
            .unwrap();
        let marker = object.get(js_string!("marker"), context)?;
        let marker = if marker.is_undefined() {
            None
        } else {
            Some(marker.to_string(context).unwrap().to_std_string_escaped())
        };

        let select = if object.has_own_property(js_string!("select"), context)? {
            Some(object.get(js_string!("select"), context)?)
        } else {
            None
        };
        let details = if object.has_own_property(js_string!("details"), context)? {
            Some(object.get(js_string!("details"), context)?)
        } else {
            None
        };

        Ok(Self {
            label,
            marker,
            select,
            details,
            index: 0,
        })
    }
}

impl<'a> menu::IntoTextMenuItem<'a, MenuAction> for TextMenuItem {
    fn to_menu_item(&'a self) -> menu::TextMenuItem<'a, MenuAction> {
        if self.label.is_empty() || self.label.chars().all(|c| c == '-') {
            menu::TextMenuItem::separator()
        } else if self.select.is_some() {
            menu::TextMenuItem::navigation_item(
                self.label.as_str(),
                self.marker.as_ref().map(|m| m.as_str()).unwrap_or_default(),
                MenuAction::Select(self.index),
            )
        } else {
            menu::TextMenuItem::unselectable(self.label.as_str())
        }
    }
}

/// Menu options being passed to [`text_menu`].
#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct UiMenuOptions {
    title: Option<String>,
    items: Vec<TextMenuItem>,
    back: Option<JsValue>,
    sort: Option<JsValue>,
    sort_label: Option<String>,
}

fn text_menu_(
    mut options: UiMenuOptions,
    ContextData(host_defined): ContextData<HostData>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let app = host_defined.app_mut();
    for (i, item) in options.items.iter_mut().enumerate() {
        item.index = i;
    }

    let mut state = menu::GolemMenuState::default();
    loop {
        let sort_label = options.sort_label.as_deref();

        let menu_options = menu::TextMenuOptions::default()
            .with_back_menu(options.back.is_some())
            .with_show_sort(options.sort.is_some())
            .with_sort_opt(sort_label)
            .with_state(Some(state));

        let (result, new_state) = menu::text_menu(
            app,
            &options.title.clone().unwrap_or_default(),
            options.items.as_slice(),
            menu_options,
        );
        state = new_state;

        fn call_callable(
            action: JsString,
            maybe_callable: &JsValue,
            context: &mut Context,
        ) -> JsResult<Option<JsValue>> {
            if let Some(callable) = maybe_callable.as_callable() {
                let result = callable.call(&JsValue::null(), &[], context)?;
                if result.is_undefined() {
                    return Ok(None);
                }
                Ok(Some(result))
            } else {
                Ok(Some(
                    JsArray::from_iter([action.into(), maybe_callable.clone()], context).into(),
                ))
            }
        }

        match result {
            MenuAction::Select(i) => {
                if let Some(ref select) = options.items[i].select {
                    if let Some(v) = call_callable(js_string!("select"), select, context)? {
                        return Ok(v);
                    }
                }
            }
            MenuAction::Details(i) => {
                if let Some(ref details) = options.items[i].details {
                    if let Some(v) = call_callable(js_string!("select"), details, context)? {
                        return Ok(v);
                    }
                }
            }
            MenuAction::Sort => {
                if let Some(ref maybe_callable) = options.sort {
                    if let Some(v) = call_callable(js_string!("sort"), maybe_callable, context)? {
                        // In sort, we try to replace partial options with the result of the callable.
                        // If this doesn't work, we return the value.
                        let Ok(mut new_options): JsResult<UiMenuOptions> = v.try_js_into(context)
                        else {
                            return Ok(v);
                        };

                        options.sort_label = new_options.sort_label.clone();
                        if let Some(new_title) = new_options.title.clone() {
                            options.title = Some(new_title);
                        }
                        std::mem::swap(&mut options.items, &mut new_options.items);
                        for (i, item) in options.items.iter_mut().enumerate() {
                            item.index = i;
                        }
                    }
                }
            }
            MenuAction::Back => {
                if let Some(ref maybe_callable) = options.back {
                    if let Some(v) = call_callable(js_string!("back"), maybe_callable, context)? {
                        return Ok(v);
                    }
                }
            }
        }
    }
}

fn alert_(
    message: String,
    title: Option<String>,
    ContextData(host_defined): ContextData<HostData>,
) {
    // Swap title and message if title is specified.
    let (message, title) = if let Some(t) = title {
        (t, message)
    } else {
        (message, "".to_string())
    };

    let app = host_defined.app_mut();
    golem_ui::application::panels::alert::alert(app, &title, &message, &["OK"]);
}

fn qr_code_(
    url: String,
    message: String,
    title: Option<String>,
    ContextData(host_defined): ContextData<HostData>,
) {
    // Swap title and message if title is specified.
    let (message, title) = if let Some(t) = title {
        (t, message)
    } else {
        (message, "".to_string())
    };

    let app = host_defined.app_mut();
    golem_ui::application::panels::qrcode::qrcode_alert(app, &title, &message, &url);
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("ui"),
        [
            (js_string!("alert"), alert_.into_js_function_copied(context)),
            (
                js_string!("qrCode"),
                qr_code_.into_js_function_copied(context),
            ),
            (
                js_string!("textMenu"),
                text_menu_.into_js_function_copied(context),
            ),
            (
                js_string!("selectFile"),
                filesystem::select.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

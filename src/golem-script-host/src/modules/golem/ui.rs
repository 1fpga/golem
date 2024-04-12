use boa_engine::{
    Context, Finalize, js_string, JsData, JsNativeError, JsResult, JsString, JsValue, Module, Trace,
};
use boa_engine::object::builtins::JsArray;
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
    selectable: bool,
    id: JsValue,
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
                    selectable: true,
                    id: JsValue::undefined(),
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

        let selectable = object
            .get(js_string!("selectable"), context)?
            .as_boolean()
            .unwrap_or(true);

        let id = object.get(js_string!("id"), context)?;

        Ok(Self {
            label,
            marker,
            selectable,
            id,
            index: 0,
        })
    }
}

impl<'a> menu::IntoTextMenuItem<'a, MenuAction> for TextMenuItem {
    fn to_menu_item(&'a self) -> menu::TextMenuItem<'a, MenuAction> {
        if self.label.is_empty() || self.label.chars().all(|c| c == '-') {
            menu::TextMenuItem::separator()
        } else if self.selectable {
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
    title: String,
    back: Option<bool>,
    items: Vec<TextMenuItem>,
}

fn text_menu_(
    mut options: UiMenuOptions,
    ContextData(host_defined): ContextData<HostData>,
    context: &mut Context,
) -> JsArray {
    let mut app = host_defined.app_mut();
    for (i, item) in options.items.iter_mut().enumerate() {
        item.index = i;
    }

    let mut state = menu::GolemMenuState::default();
    loop {
        let menu_options = menu::TextMenuOptions::default()
            .with_back_menu(options.back.unwrap_or(true))
            .with_state(Some(state));

        let (result, new_state) = menu::text_menu(
            &mut app,
            &options.title,
            options.items.as_slice(),
            menu_options,
        );
        state = new_state;

        return match result {
            MenuAction::Select(i) => JsArray::from_iter(
                [js_string!("select").into(), options.items[i].id.clone()],
                context,
            ),
            MenuAction::Details(i) => {
                let value: JsValue = options.items[i].id.clone();
                JsArray::from_iter([js_string!("details").into(), value], context)
            }
            MenuAction::Sort => JsArray::from_iter([js_string!("sort").into()], context),
            MenuAction::Back => JsArray::from_iter([js_string!("back").into()], context),
        };
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

    let mut app = host_defined.app_mut();
    golem_ui::application::panels::alert::alert(&mut app, &title, &message, &["OK"]);
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

    let mut app = host_defined.app_mut();
    golem_ui::application::panels::qrcode::qrcode_alert(&mut app, &title, &message, &url);
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

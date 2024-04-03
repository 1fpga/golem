use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use boa_engine::{
    Context, Finalize, js_string, JsData, JsNativeError, JsResult, JsString, JsValue,
    Module, Trace,
};
use boa_engine::object::builtins::JsArray;
use boa_interop::{IntoJsFunctionUnsafe, IntoJsModule};
use boa_macros::TryFromJs;

use golem_ui::application::menu::{
    GolemMenuState, IntoTextMenuItem, text_menu, TextMenuItem, TextMenuOptions,
};
use golem_ui::application::menu::style::MenuReturn;

use crate::utils::JsVec;

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuAction {
    Select(usize),
    Details(usize),
    Sort,
    Idle,
    Back,
}

impl MenuReturn for MenuAction {
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
pub struct MenuItem {
    label: String,
    marker: Option<String>,
    selectable: bool,
    id: JsValue,
    index: usize,
}

impl boa_engine::value::TryFromJs for MenuItem {
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

impl<'a> IntoTextMenuItem<'a, MenuAction> for MenuItem {
    fn to_menu_item(&'a self) -> TextMenuItem<'a, MenuAction> {
        if self.label.is_empty() || self.label.chars().all(|c| c == '-') {
            TextMenuItem::separator()
        } else {
            TextMenuItem::navigation_item(
                self.label.as_str(),
                self.marker.as_ref().map(|m| m.as_str()).unwrap_or_default(),
                MenuAction::Select(self.index),
            )
        }
    }
}

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct UiMenuOptions {
    title: String,
    back: Option<bool>,
    items: JsVec<MenuItem>,
}

fn menu(
    mut options: UiMenuOptions,
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsValue {
    // Replace missing IDs by their index.
    for (i, item) in options.items.0.iter_mut().enumerate() {
        item.index = i;
    }

    let mut state = GolemMenuState::default();
    loop {
        let menu_options: TextMenuOptions<MenuAction> = TextMenuOptions::default()
            .with_back_menu(options.back.unwrap_or(true))
            .with_state(Some(state));

        let (result, new_state) = text_menu(
            app,
            &options.title,
            options.items.0.as_slice(),
            menu_options,
        );
        state = new_state;

        match result {
            MenuAction::Select(i) => {
                let value: JsValue = options.items.0[i].id.clone();
                return
                    JsArray::from_iter([JsValue::from(js_string!("select")), value], ctx).into();
            }
            MenuAction::Details(i) => {
                return JsArray::from_iter(
                    [JsValue::from(js_string!("details")), JsValue::new(i)],
                    ctx,
                )
                    .into();
            }
            MenuAction::Sort => {
                return JsArray::from_iter([JsValue::from(js_string!("sort"))], ctx).into();
            }
            MenuAction::Back => {
                return JsArray::from_iter(
                    [JsValue::from(js_string!("back"))],
                    ctx,
                )
                    .into();
            }
            _ => {}
        }
    }
}

fn alert(
    message: JsString,
    title: Option<JsString>,
    _ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) {
    // Swap title and message if title is specified.
    let (message, title) = if let Some(t) = title {
        (t.to_std_string_escaped(), message.to_std_string_escaped())
    } else {
        (message.to_std_string_escaped(), "".to_string())
    };

    golem_ui::application::panels::alert::alert(app, &title, &message, &["OK"]);
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    unsafe {
        let menu = {
            let app = app.clone();
            move |options: UiMenuOptions, context: &mut Context| {
                menu(options, context, app.borrow_mut().deref_mut())
            }
        }.into_js_function_unsafe(context);

        let alert = {
            let app = app.clone();
            move |title, message| {
                alert(title, message, context, &mut app.borrow_mut());
                JsValue::undefined()
            }
        }.into_js_function_unsafe(context);

        Ok((
            js_string!("ui"),
            [
                (js_string!("menu"), menu),
                (js_string!("alert"), alert),
            ].into_js_module(context),
        ))
    }
}

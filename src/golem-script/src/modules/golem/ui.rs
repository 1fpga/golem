use crate::utils::JsVec;
use boa_engine::object::builtins::JsArray;
use boa_engine::{
    js_string, Context, Finalize, JsData, JsError, JsNativeError, JsResult, JsString, JsValue,
    Module, Trace,
};
use boa_macros::TryFromJs;
use golem_ui::application::menu::style::MenuReturn;
use golem_ui::application::menu::{
    text_menu, GolemMenuState, IntoTextMenuItem, TextMenuItem, TextMenuOptions,
};
use std::cell::RefCell;
use std::rc::Rc;

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
                    .into())
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
    back: bool,
    items: JsVec<MenuItem>,
}

fn menu(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let Some(options) = args.get(0) else {
        return Err(JsError::from_opaque(
            js_string!("No options provided").into(),
        ));
    };

    let mut options = options.try_js_into::<UiMenuOptions>(ctx)?;
    // Replace missing IDs by their index.
    for (i, item) in options.items.0.iter_mut().enumerate() {
        item.index = i;
    }

    let mut state = GolemMenuState::default();
    loop {
        let menu_options: TextMenuOptions<MenuAction> =
            TextMenuOptions::default().with_back_menu(options.back);

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
                return Ok(
                    JsArray::from_iter([JsValue::from(js_string!("select")), value], ctx).into(),
                );
            }
            MenuAction::Details(i) => {
                return Ok(JsArray::from_iter(
                    [JsValue::from(js_string!("details")), JsValue::new(i)],
                    ctx,
                )
                .into());
            }
            MenuAction::Sort => {
                return Ok(JsValue::from(js_string!("sort")));
            }
            MenuAction::Back => {
                return Ok(JsValue::from(js_string!("back")));
            }
            _ => {}
        }
    }
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    let menu = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_copy_closure({
            let app = Rc::downgrade(&app).as_ptr();
            move |_this, args, ctx| menu(_this, args, ctx, unsafe { &mut (*app).borrow_mut() })
        }),
    )
    .name(js_string!("menu"))
    .build();

    Ok((
        js_string!("ui"),
        Module::synthetic(
            // Make sure to list all exports beforehand.
            &[js_string!("menu")],
            // The initializer is evaluated every time a module imports this synthetic module,
            // so we avoid creating duplicate objects by capturing and cloning them instead.
            boa_engine::module::SyntheticModuleInitializer::from_copy_closure_with_captures(
                |module, fns, _| {
                    module.set_export(&js_string!("menu"), fns.0.clone().into())?;

                    Ok(())
                },
                (menu,),
            ),
            None,
            context,
        ),
    ))
}

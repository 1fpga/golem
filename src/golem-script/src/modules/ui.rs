use crate::utils::JsVec;
use boa_engine::object::builtins::JsArray;
use boa_engine::{
    js_string, Context, Finalize, JsArgs, JsData, JsError, JsNativeError, JsResult, JsString,
    JsValue, Module, Trace,
};
use boa_macros::TryFromJs;
use golem_ui::application::menu::style::MenuReturn;
use golem_ui::application::menu::{text_menu, IntoTextMenuItem, TextMenuItem, TextMenuOptions};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuAction {
    Select(i32),
    Details(i32),
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
    id: Option<i32>,
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
                    id: None,
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
        let marker = object
            .get(js_string!("marker"), context)?
            .to_string(context)
            .ok()
            .map(|s| s.to_std_string().unwrap());
        let selectable = object
            .get(js_string!("selectable"), context)?
            .as_boolean()
            .unwrap_or(true);

        let id = object.get(js_string!("id"), context)?;
        let id = id.to_i32(context).ok();

        Ok(Self {
            label,
            marker,
            selectable,
            id,
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
                MenuAction::Select(self.id.unwrap_or(0)),
            )
        }
    }
}

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct UiMenuOptions {
    title: String,
    items: JsVec<MenuItem>,
}

fn menu(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let options = args.get_or_undefined(0);
    if options.is_undefined() {
        return Err(JsError::from_opaque(
            js_string!("No options provided").into(),
        ));
    }

    let mut options = options.try_js_into::<UiMenuOptions>(ctx)?;
    // Replace missing IDs by their index.
    for (i, item) in options.items.0.iter_mut().enumerate() {
        if item.id.is_none() {
            item.id = Some(i as i32);
        }
    }

    loop {
        let (result, new_state) = text_menu(
            app,
            &options.title,
            options.items.0.as_slice(),
            TextMenuOptions::default(),
        );

        match result {
            MenuAction::Select(i) => {
                return Ok(JsArray::from_iter(
                    [JsValue::from(js_string!("select")), JsValue::new(i)],
                    ctx,
                )
                .into());
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
) -> Result<(JsString, Module), Box<dyn std::error::Error>> {
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

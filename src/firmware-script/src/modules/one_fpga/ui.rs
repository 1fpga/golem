use crate::commands::maybe_call_command;
use crate::HostData;
use boa_engine::object::builtins::{JsArray, JsPromise};
use boa_engine::value::TryFromJs;
use boa_engine::{
    js_string, Context, Finalize, JsData, JsError, JsNativeError, JsObject, JsResult, JsString,
    JsValue, Module, Trace, TryIntoJsResult,
};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use either::Either;
use firmware_ui::application::menu;
use firmware_ui::application::panels::password::enter_password;
use firmware_ui::application::panels::prompt::prompt;

mod filesystem;

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuAction {
    Select(usize),
    Details(usize),
    Sort,
    Back,
    Noop,
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

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct TextMenuItem {
    label: String,
    marker: Option<String>,
    select: Option<JsValue>,
    details: Option<JsValue>,
    index: usize,
}

impl TryFromJs for TextMenuItem {
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
            Some(marker.to_string(context)?.to_std_string_escaped())
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
            menu::TextMenuItem::unselectable_with_marker(
                self.label.as_str(),
                self.marker.as_ref().map(|m| m.as_str()).unwrap_or_default(),
            )
        }
    }
}

impl TextMenuItem {
    fn try_js_into(self, context: &mut Context) -> JsResult<JsValue> {
        let object = JsObject::with_null_proto();
        object.set(
            js_string!("label"),
            JsValue::String(self.label.clone().into()),
            false,
            context,
        )?;
        if let Some(ref marker) = self.marker {
            object.set(
                js_string!("marker"),
                JsValue::String(marker.clone().into()),
                false,
                context,
            )?;
        }
        if let Some(ref select) = self.select {
            object.set(js_string!("select"), select.clone(), false, context)?;
        }
        if let Some(ref details) = self.details {
            object.set(js_string!("details"), details.clone(), false, context)?;
        }

        object.try_into_js_result(context)
    }
}

/// Menu options being passed to [`text_menu`].
#[derive(Debug, Trace, Finalize, JsData, boa_macros::TryFromJs)]
struct UiMenuOptions {
    title: Option<String>,
    items: Vec<TextMenuItem>,
    back: Option<JsValue>,
    sort: Option<JsValue>,
    sort_label: Option<String>,
    details: Option<String>,

    highlighted: Option<u32>,

    /// If this is Some(...) do not show the menu but select the item with the given label
    /// right await (first label being found).
    #[unsafe_ignore_trace]
    selected: Option<Either<String, u32>>,
}

fn text_menu_(
    mut options: UiMenuOptions,
    ContextData(host_defined): ContextData<HostData>,
    mut context: &mut Context,
) -> JsResult<JsPromise> {
    let app = host_defined.app_mut();

    let mut state = menu::OneFpgaMenuState::default();
    loop {
        for (i, item) in options.items.iter_mut().enumerate() {
            item.index = i;
        }

        let sort_label = options.sort_label.as_deref();
        let details_label = options.details.as_deref();

        let menu_options = menu::TextMenuOptions::default()
            .with_back_menu(options.back.is_some())
            .with_show_sort(options.sort.is_some())
            .with_details_opt(details_label)
            .with_sort_opt(sort_label)
            .with_state(Some(state))
            .with_selected_opt(options.highlighted);

        let result = if let Some(label) = options.selected.take() {
            let selected_idx = match label {
                Either::Left(label) => options.items.iter().position(|i| i.label == label),
                Either::Right(idx) => {
                    if idx > options.items.len() as u32 {
                        None
                    } else {
                        Some(idx as usize)
                    }
                }
            };

            if let Some(selected_idx) = selected_idx {
                MenuAction::Select(selected_idx)
            } else {
                MenuAction::Noop
            }
        } else {
            let (result, new_state) = menu::text_menu(
                app,
                &options.title.clone().unwrap_or_default(),
                options.items.as_slice(),
                menu_options,
                &mut (host_defined.command_map_mut(), &mut context),
                |app, id, (command_map, context)| -> JsResult<()> {
                    maybe_call_command(app, id, *command_map, *context)
                },
            )?;
            state = new_state;
            result
        };

        fn call_callable(
            item: Option<(&mut TextMenuItem, usize)>,
            maybe_callable: JsValue,
            context: &mut Context,
        ) -> JsResult<Option<JsValue>> {
            if let Some(callable) = maybe_callable.as_callable() {
                let result = if let Some((item, index)) = item {
                    let js_item = item.clone().try_js_into(context)?;
                    let js_index = JsValue::from(index);

                    let mut result = callable.call(
                        &JsValue::undefined(),
                        &[js_item.clone(), js_index],
                        context,
                    )?;
                    while let Some(p) = result.as_promise() {
                        result = p.await_blocking(context).map_err(JsError::from_opaque)?;
                    }

                    if let Ok(new_item) = TryFromJs::try_from_js(&js_item, context) {
                        *item = new_item;
                    }

                    result
                } else {
                    let result = callable.call(&JsValue::undefined(), &[], context)?;
                    if let Some(p) = result.as_promise() {
                        p.await_blocking(context).map_err(JsError::from_opaque)?
                    } else {
                        result
                    }
                };

                if result.is_undefined() {
                    Ok(None)
                } else {
                    Ok(Some(result))
                }
            } else {
                Ok(Some(maybe_callable))
            }
        }

        match result {
            MenuAction::Select(i) => {
                if let Some(select) = options.items[i].select.clone() {
                    if let Some(v) =
                        call_callable(Some((&mut options.items[i], i)), select, context)?
                    {
                        return Ok(JsPromise::resolve(v, context));
                    }
                }
            }
            MenuAction::Details(i) => {
                if let Some(details) = options.items[i].details.clone() {
                    if let Some(v) =
                        call_callable(Some((&mut options.items[i], i)), details, context)?
                    {
                        return Ok(JsPromise::resolve(v, context));
                    }
                }
            }
            MenuAction::Sort => {
                if let Some(maybe_callable) = options.sort.clone() {
                    if let Some(mut result) = call_callable(None, maybe_callable, context)? {
                        if let Some(p) = result.as_promise() {
                            result = p.await_blocking(context).map_err(JsError::from_opaque)?;
                        }

                        // In sort, we try to replace partial options with the result of the callable.
                        // If this doesn't work, we return the value.
                        let Ok(mut new_options): JsResult<UiMenuOptions> =
                            result.try_js_into(context)
                        else {
                            return Ok(JsPromise::resolve(result, context));
                        };

                        options.sort_label = new_options.sort_label.clone();
                        if let Some(new_title) = new_options.title.clone() {
                            options.title = Some(new_title);
                        }
                        std::mem::swap(&mut options.items, &mut new_options.items);
                    }
                }
            }
            MenuAction::Back => {
                if let Some(maybe_callable) = options.back.clone() {
                    if let Some(v) = call_callable(None, maybe_callable, context)? {
                        return Ok(JsPromise::resolve(v, context));
                    }
                }
            }
            MenuAction::Noop => {}
        }
    }
}

#[derive(Debug, TryFromJs)]
struct AlertOptions {
    message: String,
    title: Option<String>,
    choices: Option<Vec<String>>,
}

fn alert_(
    message: Either<String, AlertOptions>,
    title: Option<String>,
    ContextData(host_defined): ContextData<HostData>,
    context: &mut Context,
) -> JsPromise {
    let (message, title, choices) = match message {
        Either::Left(message) => {
            if let Some(real_message) = title {
                (real_message, message, vec!["OK".to_string()])
            } else {
                (message, "".to_string(), vec!["OK".to_string()])
            }
        }
        Either::Right(AlertOptions {
            message,
            title,
            choices: options,
            ..
        }) => (
            message,
            title.unwrap_or_default(),
            options.unwrap_or_else(|| vec!["OK".to_string()]),
        ),
    };

    let app = host_defined.app_mut();
    let result = firmware_ui::application::panels::alert::alert(
        app,
        &title,
        &message,
        &choices.iter().map(String::as_str).collect::<Vec<_>>(),
    );

    JsPromise::resolve(
        result.map_or(JsValue::null(), |n| JsValue::from(n)),
        context,
    )
}

#[derive(Clone, Debug, Trace, Finalize, JsData, boa_macros::TryFromJs)]
pub struct PromptOptions {
    message: String,
    title: Option<String>,
    default: Option<String>,
}

fn prompt_(
    message: Either<String, PromptOptions>,
    maybe_message: Option<String>,
    ContextData(data): ContextData<HostData>,
    context: &mut Context,
) -> JsPromise {
    let (message, title, default) = match message {
        Either::Left(ref message) => {
            // Swap title and message if title is specified.
            if let Some(ref m) = maybe_message {
                (m, Some(message), None)
            } else {
                (message, None, None)
            }
        }
        Either::Right(PromptOptions {
            ref message,
            ref title,
            ref default,
        }) => (message, title.as_ref(), default.clone()),
    };

    let app = data.app_mut();
    let result = prompt(
        title.map(String::as_str).unwrap_or(""),
        message,
        default.unwrap_or_default(),
        512,
        app,
    )
    .map(|r| JsString::from(r));
    JsPromise::resolve(result.map_or(JsValue::null(), JsValue::from), context)
}

fn prompt_password_(
    title: String,
    message: String,
    length: u8,
    ContextData(data): ContextData<HostData>,
    context: &mut Context,
) -> JsPromise {
    let app = data.app_mut();
    let result = enter_password(app, &title, &message, length);

    if let Some(result) = result {
        JsPromise::resolve(
            JsArray::from_iter(
                result.iter().map(|i| JsString::from(i.to_string()).into()),
                context,
            ),
            context,
        )
    } else {
        JsPromise::resolve(JsValue::null(), context)
    }
}

fn show_(message: String, title: Option<String>, ContextData(host_defined): ContextData<HostData>) {
    // Swap title and message if title is specified.
    let (message, title) = if let Some(t) = title {
        (t, message)
    } else {
        (message, "".to_string())
    };

    let app = host_defined.app_mut();
    firmware_ui::application::panels::alert::show(app, &title, &message);
}

fn qr_code_(
    url: String,
    message: String,
    title: Option<String>,
    ContextData(host_defined): ContextData<HostData>,
) {
    // Swap title and message if title is specified. This is a JavaScript function,
    // so we need to do this here.
    let (message, title) = if let Some(t) = title {
        (t, message)
    } else {
        (message, "".to_string())
    };

    let app = host_defined.app_mut();
    firmware_ui::application::panels::qrcode::qrcode_alert(app, &title, &message, &url);
}

fn input_tester_(ContextData(host_defined): ContextData<HostData>, ctx: &mut Context) -> JsPromise {
    let app = host_defined.app_mut();
    firmware_ui::application::panels::input_tester::input_tester(app);

    JsPromise::resolve(JsValue::undefined(), ctx)
}

fn prompt_shortcut_(
    ContextData(host_defined): ContextData<HostData>,
    title: Option<String>,
    message: Option<String>,
    context: &mut Context,
) -> JsPromise {
    let app = host_defined.app_mut();
    let result = firmware_ui::application::panels::shortcut::prompt_shortcut(
        app,
        title.unwrap_or("Pick a shortcut".to_string()).as_str(),
        message.as_deref(),
    );

    if let Some(result) = result {
        JsPromise::resolve(JsString::from(result.to_string()), context)
    } else {
        JsPromise::resolve(JsValue::undefined(), context)
    }
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("ui"),
        [
            (js_string!("alert"), alert_.into_js_function_copied(context)),
            (
                js_string!("prompt"),
                prompt_.into_js_function_copied(context),
            ),
            (
                js_string!("promptPassword"),
                prompt_password_.into_js_function_copied(context),
            ),
            (
                js_string!("qrCode"),
                qr_code_.into_js_function_copied(context),
            ),
            (js_string!("show"), show_.into_js_function_copied(context)),
            (
                js_string!("textMenu"),
                text_menu_.into_js_function_copied(context),
            ),
            (
                js_string!("selectFile"),
                filesystem::select.into_js_function_copied(context),
            ),
            (
                js_string!("inputTester"),
                input_tester_.into_js_function_copied(context),
            ),
            (
                js_string!("promptShortcut"),
                prompt_shortcut_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

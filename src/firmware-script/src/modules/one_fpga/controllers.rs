use crate::HostData;
use boa_engine::{js_string, Context, JsError, JsResult, JsString, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};

fn load_mapping_(ContextData(data): ContextData<HostData>, mapping: JsString) -> JsResult<()> {
    let app = data.app_mut();

    let mapping = mapping.to_std_string_lossy();
    let mapping = mapping.as_str();

    app.platform_mut()
        .platform
        .gamepad
        .borrow_mut()
        .add_mapping(mapping)
        .map_err(JsError::from_rust)?;

    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    let module = [(
        js_string!("loadMapping"),
        load_mapping_.into_js_function_copied(context),
    )]
    .into_js_module(context);

    Ok((js_string!("controllers"), module))
}

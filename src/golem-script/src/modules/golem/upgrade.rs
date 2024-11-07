use boa_engine::object::builtins::JsUint8Array;
use boa_engine::{js_string, Context, JsResult, JsString, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};

fn upgrade_(path: String, signature: Option<JsUint8Array>, context: &mut Context) -> JsResult<()> {
    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("upgrade"),
        [(
            js_string!("upgrade"),
            upgrade_.into_js_function_copied(context),
        )]
        .into_js_module(context),
    ))
}

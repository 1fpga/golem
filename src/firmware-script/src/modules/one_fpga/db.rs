use crate::modules::one_fpga::globals::classes::JsDb;
use boa_engine::class::Class;
use boa_engine::{js_string, Context, Finalize, JsObject, JsResult, JsString, Module, Trace};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use boa_macros::TryFromJs;

#[derive(Debug, TryFromJs, Finalize, Trace)]
struct LoadOptions {
    migrations: Option<JsString>,
}

fn load_(
    name: JsString,
    _options: Option<LoadOptions>,
    context: &mut Context,
) -> JsResult<JsObject> {
    JsDb::from_data(JsDb::new(&name.to_std_string_lossy())?, context)
}

fn reset_(name: JsString) -> JsResult<()> {
    JsDb::reset(&name.to_std_string_lossy())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    let module =
        [
            (js_string!("load"), load_.into_js_function_copied(context)),
            (js_string!("reset"), reset_.into_js_function_copied(context)),
        ].into_js_module(context);

    Ok((js_string!("db"), module))
}

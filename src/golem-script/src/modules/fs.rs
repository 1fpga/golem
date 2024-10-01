use crate::module_loader::GolemModuleLoader;
use boa_engine::object::builtins::{JsArrayBuffer, JsPromise, JsUint8Array};
use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, JsValue};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use std::path::PathBuf;
use std::rc::Rc;

fn write_file(file: JsString, data: JsValue, context: &mut Context) -> JsResult<JsPromise> {
    let path = PathBuf::from(file.to_std_string_escaped());

    let data = if let Some(s) = data.as_string() {
        Ok::<_, JsError>(s.to_std_string_escaped().into_bytes())
    } else if let Some(o) = data.as_object() {
        if let Ok(buffer) = JsArrayBuffer::from_object(o.clone()) {
            Ok(JsUint8Array::from_array_buffer(buffer, context)?
                .iter(context)
                .collect())
        } else if let Ok(array) = JsUint8Array::from_object(o.clone()) {
            Ok(array.iter(context).collect())
        } else {
            Err(js_error!("Invalid data type"))
        }
    } else {
        Err(js_error!("Invalid data type"))
    }?;
    eprintln!("Writing to file: {:?} {:?}", path, data.len());

    let promise = JsPromise::new(
        |resolvers, context| {
            eprintln!("promise inner");

            match std::fs::write(&path, &data) {
                Ok(_) => {
                    eprintln!("promise resolve");
                    resolvers.resolve.call(&JsValue::undefined(), &[], context)
                }
                Err(e) => resolvers.reject.call(
                    &JsValue::undefined(),
                    &[js_string!(e.to_string()).into()],
                    context,
                ),
            }
        },
        context,
    );
    Ok(promise)
}

fn read_file(file: JsString, context: &mut Context) -> JsResult<JsPromise> {
    let path = PathBuf::from(file.to_std_string_escaped());

    let promise = JsPromise::new(
        |resolvers, context| match std::fs::read(&path) {
            Ok(data) => {
                let buffer = JsUint8Array::from_iter(data, context)?;
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[buffer.into()], context)
            }
            Err(e) => {
                let v: JsValue = js_error!("{}", e).to_opaque(context);
                resolvers.reject.call(&JsValue::undefined(), &[v], context)
            }
        },
        context,
    );
    Ok(promise)
}

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut Context,
) -> JsResult<()> {
    loader.insert_named(
        js_string!("@:fs"),
        [
            (
                js_string!("writeFile"),
                write_file.into_js_function_copied(context),
            ),
            (
                js_string!("readFile"),
                read_file.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    );
    Ok(())
}

use crate::module_loader::GolemModuleLoader;
use boa_engine::object::builtins::{JsArray, JsArrayBuffer, JsPromise, JsUint8Array};
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

fn read_file(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
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
    )
}

fn read_text_file(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
        |resolvers, context| match std::fs::read_to_string(&path) {
            Ok(data) => {
                let buffer = JsString::from(data);
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
    )
}

fn delete_file(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
        |resolvers, context| match std::fs::remove_file(&path) {
            Ok(_) => resolvers.resolve.call(&JsValue::undefined(), &[], context),
            Err(e) => {
                let v: JsValue = js_error!("{}", e).to_opaque(context);
                resolvers.reject.call(&JsValue::undefined(), &[v], context)
            }
        },
        context,
    )
}

fn read_dir(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
        |resolvers, context| {
            let entries = match std::fs::read_dir(&path) {
                Ok(entries) => entries,
                Err(e) => {
                    let v: JsValue = js_error!("{}", e).to_opaque(context);
                    return resolvers.reject.call(&JsValue::undefined(), &[v], context);
                }
            };

            let entries = entries
                .filter_map(|entry| {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            let v: JsValue = js_error!("{}", e).to_opaque(context);
                            return Some(v);
                        }
                    };
                    let path = entry.path();
                    let path = JsString::from(path.to_string_lossy().as_ref());
                    Some(path.into())
                })
                .collect::<Vec<JsValue>>();
            resolvers.resolve.call(
                &JsValue::undefined(),
                &[JsArray::from_iter(entries, context).into()],
                context,
            )
        },
        context,
    )
}

fn is_file(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
        |resolvers, context| {
            let is_file = match std::fs::metadata(&path) {
                Ok(metadata) => metadata.is_file(),
                Err(_) => false,
            };
            resolvers.resolve.call(
                &JsValue::undefined(),
                &[JsValue::from(is_file).into()],
                context,
            )
        },
        context,
    )
}

fn is_dir(file: JsString, context: &mut Context) -> JsPromise {
    let path = PathBuf::from(file.to_std_string_escaped());

    JsPromise::new(
        |resolvers, context| {
            let is_dir = match std::fs::metadata(&path) {
                Ok(metadata) => metadata.is_dir(),
                Err(_) => false,
            };
            resolvers.resolve.call(
                &JsValue::undefined(),
                &[JsValue::from(is_dir).into()],
                context,
            )
        },
        context,
    )
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
            (
                js_string!("readTextFile"),
                read_text_file.into_js_function_copied(context),
            ),
            (
                js_string!("deleteFile"),
                delete_file.into_js_function_copied(context),
            ),
            (
                js_string!("readDir"),
                read_dir.into_js_function_copied(context),
            ),
            (
                js_string!("isFile"),
                is_file.into_js_function_copied(context),
            ),
            (js_string!("isDir"), is_dir.into_js_function_copied(context)),
        ]
        .into_js_module(context),
    );
    Ok(())
}

use boa_engine::object::builtins::{JsArray, JsArrayBuffer, JsPromise, JsUint8Array};
use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use boa_macros::TryFromJs;
use either::Either;
use sha2::Digest;
use std::path::PathBuf;

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

    let promise = JsPromise::new(
        |resolvers, context| {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(JsError::from_rust)?;
            std::fs::write(&path, &data).map_err(JsError::from_rust)?;
            resolvers.resolve.call(&JsValue::undefined(), &[], context)
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

#[derive(Debug, TryFromJs)]
struct FindAllFilesOptions {
    extensions: Option<Vec<String>>,
}

fn find_all_files(
    root: String,
    options: Option<FindAllFilesOptions>,
    context: &mut Context,
) -> JsPromise {
    JsPromise::new(
        |fns, context| {
            let extensions = options
                .as_ref()
                .and_then(|o| o.extensions.as_ref())
                .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>());

            let files = walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    if !path.is_file() {
                        return None;
                    }

                    if let Some(extensions) = &extensions {
                        if let Some(ext) = path.extension() {
                            if !extensions.iter().any(|e| *e == ext) {
                                return None;
                            }
                        }
                    }
                    let path = JsString::from(path.to_string_lossy().as_ref());
                    Some(path.into())
                })
                .collect::<Vec<JsValue>>();

            fns.resolve.call(
                &JsValue::undefined(),
                &[JsArray::from_iter(files, context).into()],
                context,
            )?;

            Ok(JsValue::undefined())
        },
        context,
    )
}

fn sha256(paths: Either<String, Vec<String>>, context: &mut Context) -> JsPromise {
    JsPromise::new(
        |fns, context| {
            let value: JsValue = match paths {
                Either::Left(path) => {
                    let path = PathBuf::from(path);
                    let data = std::fs::read(&path).unwrap();
                    let hash = sha2::Sha256::digest(&data);
                    let hash = hash
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    JsString::from(hash).into()
                }
                Either::Right(paths) => {
                    let hashes = paths
                        .into_iter()
                        .map(|path| {
                            let path = PathBuf::from(path);
                            let data = std::fs::read(&path).unwrap();
                            let hash = sha2::Sha256::digest(&data);
                            let hash = hash
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>();
                            JsString::from(hash).into()
                        })
                        .collect::<Vec<JsValue>>();
                    JsArray::from_iter(hashes, context).into()
                }
            };

            fns.resolve.call(&JsValue::undefined(), &[value], context)?;
            Ok(JsValue::undefined())
        },
        context,
    )
}

fn file_size(paths: Either<String, Vec<String>>, context: &mut Context) -> JsPromise {
    JsPromise::new(
        |fns, context| {
            let value: JsValue = match paths {
                Either::Left(path) => {
                    let path = PathBuf::from(path);
                    let size = std::fs::metadata(&path).unwrap().len();
                    JsValue::from(size).into()
                }
                Either::Right(paths) => {
                    let hashes = paths
                        .into_iter()
                        .map(|path| {
                            let path = PathBuf::from(path);
                            let size = std::fs::metadata(&path).unwrap().len();
                            JsValue::from(size).into()
                        })
                        .collect::<Vec<JsValue>>();
                    JsArray::from_iter(hashes, context).into()
                }
            };

            fns.resolve.call(&JsValue::undefined(), &[value], context)?;
            Ok(JsValue::undefined())
        },
        context,
    )
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    let module = [
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
        (
            js_string!("findAllFiles"),
            find_all_files.into_js_function_copied(context),
        ),
        (
            js_string!("sha256"),
            sha256.into_js_function_copied(context),
        ),
        (
            js_string!("fileSize"),
            file_size.into_js_function_copied(context),
        ),
    ]
    .into_js_module(context);

    Ok((js_string!("fs"), module))
}

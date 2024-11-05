use std::path::Path;

use crate::commands::maybe_call_command;
use crate::HostData;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::TryFromJs;
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, JsValue, TryIntoJsResult};
use boa_interop::ContextData;
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Finalize, Trace, JsData, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectFileOptions {
    allow_back: Option<bool>,
    dir_first: Option<bool>,
    show_hidden: Option<bool>,
    show_extensions: Option<bool>,
    directory: Option<bool>,
    filter_pattern: Option<String>,
    extensions: Option<Vec<String>>,
}

impl TryFromJs for SelectFileOptions {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let serde_v = value.to_json(context)?;
        let options: SelectFileOptions = serde_json::from_value(serde_v)
            .map_err(|e| JsError::from(JsNativeError::typ().with_message(e.to_string())))?;

        Ok(options)
    }
}

impl TryInto<FilesystemMenuOptions> for SelectFileOptions {
    type Error = String;

    fn try_into(mut self) -> Result<FilesystemMenuOptions, Self::Error> {
        let pattern = match &self.filter_pattern {
            Some(p) => Some(Regex::new(p).map_err(|e| e.to_string())?),
            None => None,
        };
        let extensions = self.extensions.take();

        Ok(FilesystemMenuOptions {
            allow_back: self.allow_back,
            dir_first: self.dir_first,
            show_hidden: self.show_hidden,
            show_extensions: self.show_extensions,
            directory: self.directory,
            pattern,
            extensions,
        })
    }
}

pub fn select(
    title: String,
    initial_dir: String,
    options: SelectFileOptions,
    ContextData(data): ContextData<HostData>,
    context: &mut Context,
) -> JsPromise {
    JsPromise::new(
        move |fns, mut context| {
            let command_map = data.command_map_mut();
            let app = data.app_mut();

            let result = options
                .try_into()
                .and_then(|options| {
                    select_file_path_menu(
                        app,
                        &title,
                        Path::new(&initial_dir),
                        options,
                        &mut (command_map, &mut context),
                        |app, id, (command_map, context)| -> JsResult<()> {
                            maybe_call_command(app, id, *command_map, *context)
                        },
                    )
                    .map_err(|e| e.to_string())
                })
                .map(|path| path.map(|p| JsString::from(p.to_string_lossy().to_string())))
                .map_err(|e| JsError::from_opaque(JsString::from(e).into()));
            let result = result.try_into_js_result(context);

            match result {
                Ok(v) => fns.resolve.call(&JsValue::Undefined, &[v], context),
                Err(e) => fns
                    .reject
                    .call(&JsValue::Undefined, &[e.to_opaque(context)], context),
            }
        },
        context,
    )
}

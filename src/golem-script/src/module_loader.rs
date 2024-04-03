use std::collections::HashMap;
use std::path::{Path, PathBuf};

use boa_engine::{Context, js_string, JsError, JsNativeError, JsResult, JsString, Module, Source};
use boa_engine::module::{ModuleLoader, Referrer};
use boa_gc::GcRefCell;

/// A module loader that also understands "free-standing" modules and
/// special resolution.
pub struct GolemModuleLoader {
    root: PathBuf,
    named_module_map: GcRefCell<HashMap<JsString, Module>>,
    cache: GcRefCell<HashMap<PathBuf, Module>>,
}

impl GolemModuleLoader {
    /// Creates a new `GolemModuleLoader` from a root module path without checking
    /// the path exists.
    fn new_unchecked(root: PathBuf) -> Self {
        Self {
            root,
            named_module_map: GcRefCell::default(),
            cache: GcRefCell::default(),
        }
    }

    /// Creates a new `GolemModuleLoader` from a root module path.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        Ok(Self::new_unchecked(root.into().canonicalize()?))
    }

    /// Inserts a module in the named module map.
    #[inline]
    pub fn insert_named(&self, name: JsString, module: Module) {
        self.named_module_map.borrow_mut().insert(name, module);
    }

    /// Inserts a new module onto the module map.
    #[inline]
    pub fn insert(&self, path: PathBuf, module: Module) {
        self.cache.borrow_mut().insert(path, module);
    }

    /// Gets a module from its original path.
    #[inline]
    pub fn get(&self, path: &PathBuf) -> Option<Module> {
        self.cache.borrow().get(path).cloned()
    }
}

impl ModuleLoader for GolemModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let result = (|| {
            // First, try to resolve from our internal cached.
            if let Some(module) = self.named_module_map.borrow().get(&specifier) {
                return Ok(module.clone());
            }

            // Otherwise, try to resolve using the file system.
            let path = specifier
                .to_std_string()
                .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
            let path = boa_interop::loaders::predicate::predicates::path_resolver(
                self.root.clone(),
            )(referrer.path(), js_string!(path))?
                .to_std_string_escaped();
            let short_path = Path::new(&path);
            let path = self.root.join(short_path);
            let path = path.canonicalize().map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!(
                        "could not canonicalize path `{}`",
                        short_path.display()
                    ))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            if let Some(module) = self.get(&path) {
                return Ok(module);
            }

            let source = Source::from_filepath(&path).map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("could not open file `{}`", short_path.display()))
                    .with_cause(JsError::from_opaque(js_string!(err.to_string()).into()))
            })?;
            let module = Module::parse(source, None, context).map_err(|err| {
                JsNativeError::syntax()
                    .with_message(format!("could not parse module `{}`", short_path.display()))
                    .with_cause(err)
            })?;
            self.insert(path, module.clone());
            Ok(module)
        })();

        finish_load(result, context);
    }
}

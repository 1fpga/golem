use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::module::{ModuleLoader, Referrer, SimpleModuleLoader};
use boa_engine::{Context, JsResult, JsString, Module};
use boa_interop::embed_module;
use boa_interop::loaders::HashMapModuleLoader;

/// A module loader that also understands "freestanding" modules and
/// special resolution.
pub struct GolemModuleLoader {
    named_modules: Rc<RefCell<HashMapModuleLoader>>,
    inner: Rc<dyn ModuleLoader>,
}

impl Default for GolemModuleLoader {
    fn default() -> Self {
        Self {
            named_modules: Rc::new(RefCell::new(HashMapModuleLoader::default())),
            inner: Rc::new(embed_module!("../golem-frontend/dist/")),
        }
    }
}

impl GolemModuleLoader {
    fn new_unchecked(root: PathBuf) -> Self {
        Self {
            named_modules: Rc::new(RefCell::new(HashMapModuleLoader::default())),
            inner: Rc::new(
                SimpleModuleLoader::new(root).expect("Could not find the script folder."),
            ),
        }
    }

    /// Creates a new `GolemModuleLoader` from a root module path.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        root.into().canonicalize().map(Self::new_unchecked)
    }

    /// Inserts a module in the named module map.
    #[inline]
    pub fn insert_named(&self, name: JsString, module: Module) {
        self.named_modules.borrow_mut().register(name, module);
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
        let inner = self.inner.clone();
        self.named_modules.borrow().load_imported_module(
            referrer.clone(),
            specifier.clone(),
            Box::new(move |module, context| {
                if module.is_ok() {
                    finish_load(module, context);
                } else {
                    inner
                        .as_ref()
                        .load_imported_module(referrer, specifier, finish_load, context);
                }
            }),
            context,
        );
    }

    fn get_module(&self, specifier: JsString) -> Option<Module> {
        self.named_modules
            .borrow()
            .get_module(specifier.clone())
            .or_else(|| self.inner.as_ref().get_module(specifier))
    }
}

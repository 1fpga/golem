use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsPromise;
use boa_engine::{js_error, js_string, Context, JsObject, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, IntoJsFunctionCopied, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use firmware_ui::application::OneFpgaApp;
use mister_fpga::core::AsMisterCore;
use std::rc::Rc;
use tracing::error;

/// An image.
#[derive(Clone, Trace, Finalize, JsData)]
pub struct JsImage {
    #[unsafe_ignore_trace]
    inner: Rc<image::DynamicImage>,
}

impl JsImage {
    /// Create a new `JsImage`.
    pub fn new(inner: image::DynamicImage) -> Self {
        Self {
            inner: Rc::new(inner),
        }
    }

    pub fn load(path: &str) -> JsResult<JsImage> {
        let image = image::open(path).map_err(|e| js_error!("Failed to load image: {}", e))?;
        Ok(Self::new(image))
    }

    pub fn load_js(path: JsString, context: &mut Context) -> JsResult<JsObject> {
        Self::from_data(
            Self::load(
                path.to_std_string()
                    .map_err(|_| js_error!("Invalid path."))?
                    .as_str(),
            )?,
            context,
        )
    }

    pub fn into_object(self, context: &mut Context) -> JsResult<JsValue> {
        Self::from_data(self, context).map(JsValue::Object)
    }

    /// Get the width of the image.
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Get the height of the image.
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Put the image as the background, if on the menu core.
    pub fn send_to_background(&self, app: &mut OneFpgaApp, force: bool) -> () {
        let Some(mut maybe_core) = app.platform_mut().core_manager_mut().get_current_core() else {
            return;
        };

        let Some(maybe_menu) = maybe_core.as_menu_core_mut() else {
            return;
        };

        if let Some(image) = self.inner.as_rgb8() {
            if let Err(error) = maybe_menu.send_to_framebuffer(image) {
                error!(error, "Failed to send image to framebuffer");
            }
        } else if force {
            let image = self.inner.as_ref().clone().into_rgb8();
            if let Err(error) = maybe_menu.send_to_framebuffer(&image) {
                error!(error, "Failed to send image to framebuffer");
            }
        }
    }

    /// Save the image
    pub fn save(&self, path: String, context: &mut Context) -> JsResult<JsPromise> {
        let inner = self.inner.clone();
        let promise = JsPromise::new(
            |fns, context| match inner.save(path) {
                Ok(()) => fns.resolve.call(&JsValue::null(), &[], context),
                Err(e) => fns.reject.call(
                    &JsValue::null(),
                    &[js_error!("Failed to save image: {}", e).to_opaque(context)],
                    context,
                ),
            },
            context,
        );

        Ok(promise)
    }
}

js_class! {
    class JsImage as "Image" {
        property width {
            fn get(this: JsClass<JsImage>) -> u32 {
                this.borrow().width()
            }
        }

        property height {
            fn get(this: JsClass<JsImage>) -> u32 {
                this.borrow().height()
            }
        }

        constructor() {
            Err(js_error!("Cannot construct Image."))
        }

        init(class: &mut ClassBuilder) -> JsResult<()> {
            let load = Self::load_js.into_js_function_copied(class.context());

            class.static_method(js_string!("load"), 1, load);

            Ok(())
        }

        fn send_to_background(host_data: ContextData<HostData>, this: JsClass<JsImage>, force: bool) -> () {
            this.borrow().send_to_background(host_data.0.app_mut(), force);
        }

        fn save(this: JsClass<JsImage>, path: JsString, context: &mut Context) -> JsResult<JsPromise> {
            this.borrow()
                .save(
                    path.to_std_string().map_err(|_| js_error!("Invalid path."))?,
                    context,
                )
        }
    }
}

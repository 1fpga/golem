use boa_engine::class::Class;
use boa_engine::{js_error, Context, JsResult, JsValue};
use boa_interop::{js_class, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use std::rc::Rc;

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

    pub fn into_object(self, context: &mut Context) -> JsResult<JsValue> {
        Self::from_data(self, context).map(JsValue::Object)
    }

    /// Get the inner `DynamicImage`.
    pub fn inner(&self) -> &Rc<image::DynamicImage> {
        &self.inner
    }

    /// Get the width of the image.
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Get the height of the image.
    pub fn height(&self) -> u32 {
        self.inner.height()
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
    }
}

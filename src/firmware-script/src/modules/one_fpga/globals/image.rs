use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsPromise;
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, js_string, Context, JsObject, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, IntoJsFunctionCopied, JsClass};
use boa_macros::{js_str, Finalize, JsData, Trace};
use firmware_ui::application::OneFpgaApp;
use image::DynamicImage;
use mister_fpga::core::AsMisterCore;
use std::rc::Rc;
use tracing::error;

/// Position of the image.
#[derive(Debug, Default, Clone, Copy)]
pub enum Position {
    /// Top-left corner.
    #[default]
    TopLeft,

    /// Centered.
    Center,

    /// Specific position.
    Custom { x: i64, y: i64 },
}

impl TryFromJs for Position {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        if value.is_null_or_undefined() {
            Ok(Self::default())
        } else if let Some(v) = value.as_object() {
            let x = v.get(js_str!("x"), context)?;
            let y = v.get(js_str!("y"), context)?;

            Ok(Self::Custom {
                x: x.to_number(context)? as i64,
                y: y.to_number(context)? as i64,
            })
        } else {
            let s = value
                .to_string(context)?
                .to_std_string_lossy()
                .to_lowercase();

            Ok(match s.as_str() {
                "top-left" => Self::TopLeft,
                "center" => Self::Center,
                _ => return Err(js_error!("Invalid position")),
            })
        }
    }
}

/// Options for the sendToBackground method.
#[derive(Debug, Clone, Copy, TryFromJs)]
pub struct SendToBackgroundOptions {
    /// Clear the background first. Default to false.
    clear: Option<bool>,

    /// Position of the image.
    position: Option<Position>,
}

/// An image.
#[derive(Clone, Trace, Finalize, JsData)]
pub struct JsImage {
    #[unsafe_ignore_trace]
    inner: Rc<DynamicImage>,
}

impl JsImage {
    /// Create a new `JsImage`.
    pub fn new(inner: DynamicImage) -> Self {
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

    /**
     * Load an image from the embedded resources, by its name.
     */
    pub fn load_embedded(name: &str) -> JsResult<JsImage> {
        match name {
            "background" => {
                let image =
                    image::load_from_memory(include_bytes!("../../../../assets/background.jpg"))
                        .map_err(|e| js_error!("Failed to load embedded image: {}", e))?;
                Ok(Self::new(image))
            }
            _ => Err(js_error!("Unknown embedded image.")),
        }
    }

    pub fn load_embedded_js(path: JsString, context: &mut Context) -> JsResult<JsObject> {
        Self::from_data(
            Self::load_embedded(
                path.to_std_string()
                    .map_err(|_| js_error!("Invalid path."))?
                    .as_str(),
            )?,
            context,
        )
    }

    pub fn into_object(self, context: &mut Context) -> JsResult<JsObject> {
        Self::from_data(self, context)
    }

    /// Get the width of the image.
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Get the height of the image.
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Resize the image, returning a new image.
    pub fn resize(&self, width: u32, height: u32, ar: bool) -> Self {
        Self::new(if ar {
            self.inner
                .resize(width, height, image::imageops::FilterType::Nearest)
        } else {
            self.inner
                .resize_exact(width, height, image::imageops::FilterType::Nearest)
        })
    }

    /// Put the image as the background, if on the menu core.
    pub fn send_to_background(
        &self,
        app: &mut OneFpgaApp,
        options: Option<SendToBackgroundOptions>,
    ) -> () {
        let Some(mut maybe_core) = app.platform_mut().core_manager_mut().get_current_core() else {
            return;
        };

        let Some(maybe_menu) = maybe_core.as_menu_core_mut() else {
            return;
        };

        let Ok(fb_size) = maybe_menu.video_info().map(|info| info.fb_resolution()) else {
            return;
        };

        let image = self.inner.as_ref();
        let position = options
            .and_then(|o| o.position)
            .unwrap_or(Position::default());
        let clear = options.and_then(|o| o.clear).unwrap_or(false);

        let (width, height) = (image.width() as i64, image.height() as i64);
        let (fb_width, fb_height) = (fb_size.width as i64, fb_size.height as i64);
        let (x, y) = match position {
            Position::TopLeft => (0, 0),
            Position::Center => ((fb_width - width) / 2, (fb_height - height) / 2),
            Position::Custom { x, y } => (x, y),
        };

        if clear {
            let _ = maybe_menu.clear_framebuffer();
        }

        if let Err(error) = maybe_menu.send_to_framebuffer(image, (x, y)) {
            error!(error, "Failed to send image to framebuffer");
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
            let load_embedded = Self::load_embedded_js.into_js_function_copied(class.context());

            class.static_method(js_string!("load"), 1, load);
            class.static_method(js_string!("embedded"), 1, load_embedded);

            Ok(())
        }

        fn send_to_background as "sendToBackground"(host_data: ContextData<HostData>, this: JsClass<JsImage>, options: Option<SendToBackgroundOptions>) -> () {
            this.borrow().send_to_background(host_data.0.app_mut(), options);
        }

        fn save(this: JsClass<JsImage>, path: JsString, context: &mut Context) -> JsResult<JsPromise> {
            this.borrow()
                .save(
                    path.to_std_string().map_err(|_| js_error!("Invalid path."))?,
                    context,
                )
        }

        fn resize(this: JsClass<JsImage>, width: u32, height: u32, ar: Option<bool>, context: &mut Context) -> JsResult<JsObject> {
            this.borrow().resize(width, height, ar.unwrap_or(true)).into_object(context)
        }
    }
}

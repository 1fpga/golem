use crate::commands::maybe_call_command;
use crate::modules::one_fpga::globals::classes::JsImage;
use crate::HostData;
use boa_engine::object::builtins::{JsFunction, JsUint8Array};
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, Context, JsError, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, JsClass};
use boa_macros::{js_str, Finalize, JsData, Trace};
use enum_map::{Enum, EnumMap};
use firmware_ui::application::panels::core_loop::run_core_loop;
use firmware_ui::application::OneFpgaApp;
use mister_fpga::core::{AsMisterCore, MisterFpgaCore};
use one_fpga::core::SettingId;
use one_fpga::{Core, OneFpgaCore};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, info};

#[derive(Debug, Clone, Trace, Finalize, TryFromJs)]
struct LoopOptions {}

#[derive(Debug, Clone, Enum)]
enum Events {
    SaveState,
}

impl TryFromJs for Events {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let string = JsString::try_from_js(value, context)?;
        if string == js_str!("saveState") {
            Ok(Self::SaveState)
        } else {
            Err(js_error!(TypeError: "Unknown event type: {}", string.to_std_string_escaped()))
        }
    }
}

#[derive(Clone, Trace, Finalize, JsData)]
pub struct JsCore {
    #[unsafe_ignore_trace]
    core: OneFpgaCore,

    #[unsafe_ignore_trace]
    events: Rc<RefCell<EnumMap<Events, Vec<JsFunction>>>>,
}

impl JsCore {
    pub fn new(core: OneFpgaCore) -> Self {
        Self {
            core,
            events: Default::default(),
        }
    }

    fn reset(&mut self) -> JsResult<()> {
        self.core.reset().map_err(JsError::from_rust)
    }

    fn r#loop(
        &mut self,
        host_defined: HostData,
        options: Option<LoopOptions>,
        context: &mut Context,
    ) -> JsResult<()> {
        let app = host_defined.app_mut();
        let command_map = host_defined.command_map_mut();
        let mut core = self.core.clone();

        let events = self.events.clone();
        info!("Running loop: {:?}", options);

        run_core_loop(
            app,
            &mut core,
            &mut (command_map, context),
            |app, _core, id, (command_map, context)| -> JsResult<()> {
                maybe_call_command(app, id, command_map, context)
            },
            |_app, _core, screenshot, slot, savestate, (_, context)| {
                for handler in events.borrow()[Events::SaveState].iter() {
                    let ss = JsUint8Array::from_iter(savestate.iter().copied(), context)?;
                    let image =
                        screenshot.and_then(|i| JsImage::new(i.clone()).into_object(context).ok());
                    let result = handler.call(
                        &JsValue::undefined(),
                        &[
                            ss.into(),
                            image.unwrap_or(JsValue::undefined()),
                            JsValue::from(slot),
                        ],
                        context,
                    )?;

                    if let Some(p) = result.as_promise() {
                        p.await_blocking(context).map_err(JsError::from_opaque)?;
                    }
                }

                Ok(())
            },
        )
    }

    fn show_osd(
        &mut self,
        app: &mut OneFpgaApp,
        handler: JsFunction,
        context: &mut Context,
    ) -> JsResult<()> {
        app.platform_mut().core_manager_mut().show_osd();

        // Update saves on Mister Cores.
        if let Some(c) = self.core.as_any_mut().downcast_mut::<MisterFpgaCore>() {
            loop {
                match c.poll_mounts() {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(e) => {
                        error!(?e, "Error updating the SD card.");
                        break;
                    }
                }
            }
        }

        let mut v = handler.call(&JsValue::undefined(), &[], context)?;
        if let Some(p) = v.as_promise() {
            match p.await_blocking(context) {
                Ok(new_v) => {
                    v = new_v;
                }
                Err(e) => return Err(JsError::from_opaque(e)),
            }
        }

        if v.to_boolean() {
            self.quit();
        }

        app.platform_mut().core_manager_mut().hide_osd();
        Ok(())
    }

    fn settings(&self, context: &mut Context) -> JsResult<JsValue> {
        let settings = self.core.settings().map_err(JsError::from_rust)?;
        let json = serde_json::to_value(&settings).map_err(JsError::from_rust)?;
        JsValue::from_json(&json, context).map_err(JsError::from_rust)
    }

    fn quit(&mut self) {
        self.core.quit();
    }

    fn get_status_bits(&self, context: &mut Context) -> Option<JsUint8Array> {
        if let Some(core) = self.core.as_mister_core() {
            JsUint8Array::from_iter(
                core.status_bits().iter().map(|b| if b { 1 } else { 0 }),
                context,
            )
            .ok()
        } else {
            None
        }
    }

    fn set_status_bits(&mut self, bits: JsUint8Array, context: &mut Context) -> JsResult<()> {
        if let Some(core) = self.core.as_mister_core() {
            let mut slice = *core.status_bits();
            for bit in 0..slice.len() {
                slice.set(bit, bits.at(bit as i64, context)?.to_uint8(context)? != 0);
            }
        }
        Ok(())
    }

    fn on(&mut self, event: Events, handler: JsFunction) -> JsResult<()> {
        self.events.borrow_mut()[event].push(handler);
        Ok(())
    }
}

js_class! {
    class JsCore as "OneFpgaCore" {
        property name {
            fn get(this: JsClass<JsCore>) -> JsResult<JsString> {
                Ok(JsString::from(this.borrow().core.name()))
            }
        }

        property settings {
            fn get(this: JsClass<JsCore>, context: &mut Context) -> JsResult<JsValue> {
                this.borrow().settings(context)
            }
        }

        property status_bits as "statusBits" {
            fn get(this: JsClass<JsCore>, context: &mut Context) -> Option<JsUint8Array> {
                this.borrow().get_status_bits(context)
            }

            fn set(this: JsClass<JsCore>, value: JsUint8Array, context: &mut Context) -> JsResult<()> {
                this.borrow_mut().set_status_bits(value, context)
            }
        }

        property volume {
            fn get(this: JsClass<JsCore>) -> JsResult<f64> {
                let volume = this.borrow().core.volume().map_err(JsError::from_rust)? as f64;

                Ok(volume / 255.0)
            }

            fn set(this: JsClass<JsCore>, value: f64) -> JsResult<()> {
                let value = (value * 255.0) as u8;
                this.borrow_mut().core.set_volume(value).map_err(JsError::from_rust)
            }
        }

        constructor(data: ContextData<HostData>) {
            let host_defined = data.0;
            Ok(JsCore::new(host_defined.app_mut().platform_mut().core_manager_mut().get_current_core().unwrap().clone()))
        }

        fn reset(this: JsClass<JsCore>) -> JsResult<()> {
            this.clone_inner().reset()
        }

        fn run_loop as "loop"(
            this: JsClass<JsCore>,
            data: ContextData<HostData>,
            options: Option<LoopOptions>,
            context: &mut Context,
        ) -> JsResult<()> {
            this.clone_inner().r#loop(data.0, options, context)
        }

        fn show_osd as "showOsd"(
            this: JsClass<JsCore>,
            data: ContextData<HostData>,
            handler: JsFunction,
            context: &mut Context,
        ) -> JsResult<()> {
            this.clone_inner().show_osd(data.0.app_mut(), handler, context)
        }

        fn file_select as "fileSelect"(
            this: JsClass<JsCore>,
            id: u32,
            path: JsString,
        ) -> JsResult<()> {
            this.clone_inner().core.file_select(SettingId::from(id), path.to_std_string_lossy()).map_err(JsError::from_rust)
        }

        fn trigger(
            this: JsClass<JsCore>,
            id: u32,
        ) -> JsResult<()> {
            this.clone_inner().core.trigger(SettingId::from(id)).map_err(JsError::from_rust)
        }

        fn bool_select as "boolSelect"(
            this: JsClass<JsCore>,
            id: u32,
            value: bool,
        ) -> JsResult<bool> {
            this.clone_inner().core.bool_option(SettingId::from(id), value).map_err(JsError::from_rust)
        }

        fn int_select as "intSelect"(
            this: JsClass<JsCore>,
            id: u32,
            value: u32,
        ) -> JsResult<u32> {
            this.clone_inner().core.int_option(SettingId::from(id), value).map_err(JsError::from_rust)
        }

        fn quit(this: JsClass<JsCore>) -> () {
            this.clone_inner().quit()
        }

        fn on(
            this: JsClass<JsCore>,
            event: Events,
            handler: JsFunction,
        ) -> JsResult<()> {
            this.clone_inner().on(event, handler)
        }
    }
}

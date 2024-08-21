use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsFunction;
use boa_engine::{Context, JsError, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::panels::core_loop::run_core_loop;
use golem_ui::application::GoLEmApp;
use mister_fpga::core::MisterFpgaCore;
use one_fpga::core::SettingId;
use one_fpga::{Core, GolemCore};
use tracing::error;

#[derive(Clone, Trace, Finalize, JsData)]
pub struct JsCore {
    #[unsafe_ignore_trace]
    core: GolemCore,
}

impl JsCore {
    pub fn new(core: GolemCore) -> Self {
        Self { core }
    }

    pub fn into_object(self, context: &mut Context) -> JsResult<JsValue> {
        Self::from_data(self, context).map(JsValue::Object)
    }

    fn reset(&mut self) -> JsResult<()> {
        self.core.reset().map_err(JsError::from_rust)
    }

    fn r#loop(&mut self, host_defined: HostData, context: &mut Context) -> JsResult<()> {
        let app = host_defined.app_mut();
        let command_map = host_defined.command_map_mut();
        let mut core = self.core.clone();

        run_core_loop(
            app,
            &mut core,
            &mut (command_map, context),
            |app, core, _, id, (command_map, context)| -> JsResult<()> {
                eprintln!("Shortcut: {:?}", id);
                if let Some(command) = command_map.get_mut(id) {
                    eprintln!("Command: {:?}", command);
                    command.execute(app, Some(core), context)
                } else {
                    Ok(())
                }
            },
        )
    }

    fn show_osd(
        &mut self,
        app: &mut GoLEmApp,
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
        while let Some(p) = v.as_promise() {
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
}

js_class! {
    class JsCore as "GolemCore" {
        property settings {
            fn get(this: JsClass<JsCore>, context: &mut Context) -> JsResult<JsValue> {
                this.borrow().settings(context)
            }
        }

        property name {
            fn get(this: JsClass<JsCore>) -> JsResult<JsString> {
                Ok(JsString::from(this.borrow().core.name()))
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
            context: &mut Context,
        ) -> JsResult<()> {
            this.clone_inner().r#loop(data.0, context)
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
            this.clone_inner().core.file_select(SettingId::from(id), path.to_std_string_escaped()).map_err(JsError::from_rust)
        }

        fn trigger(
            this: JsClass<JsCore>,
            id: u32,
        ) -> JsResult<()> {
            this.clone_inner().core.trigger(SettingId::from(id)).map_err(JsError::from_rust)
        }

        fn quit(this: JsClass<JsCore>) -> () {
            this.clone_inner().quit()
        }
    }
}

use crate::HostData;
use boa_engine::class::Class;
use boa_engine::{Context, JsError, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::panels::core_loop;
use golem_ui::application::panels::core_loop::run_core_loop;
use golem_ui::application::GoLEmApp;
use one_fpga::{Core, GolemCore};

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
        self.core.reset().map_err(JsError::from_std)
    }

    fn r#loop(
        &mut self,
        host_defined: HostData,
        show_menu: bool,
        context: &mut Context,
    ) -> JsResult<()> {
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
            show_menu,
        )
    }

    fn show_menu(&mut self, app: &mut GoLEmApp) {
        if core_loop::menu::core_menu(app, &mut self.core) {
            self.quit();
        }
    }

    fn quit(&mut self) {
        self.core.quit();
    }
}

js_class! {
    class JsCore as "GolemCore" {
        constructor(data: ContextData<HostData>) {
            let host_defined = data.0;
            Ok(JsCore::new(host_defined.app_mut().platform_mut().core_manager_mut().get_current_core().unwrap().clone()))
        }

        fn reset(this: JsClass<JsCore>) -> JsResult<()> {
            this.borrow_mut().reset()
        }

        fn name(this: JsClass<JsCore>) -> JsResult<JsString> {
            Ok(JsString::from(this.borrow().core.name()))
        }

        fn run_loop as "loop"(
            this: JsClass<JsCore>,
            data: ContextData<HostData>,
            show_menu: Option<bool>,
            context: &mut Context
        ) -> JsResult<()> {
            this.borrow_mut().r#loop(data.0, show_menu.unwrap_or(false), context)
        }

        fn show_menu as "showMenu"(this: JsClass<JsCore>, data: ContextData<HostData>) -> () {
            this.borrow_mut().show_menu(data.0.app_mut())
        }

        fn quit(this: JsClass<JsCore>) -> () {
            this.borrow_mut().quit()
        }
    }
}

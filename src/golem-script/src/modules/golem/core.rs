use boa_engine::{
    js_string, Context, JsArgs, JsError, JsObject, JsResult, JsString, JsValue, Module,
};
use std::cell::RefCell;
use std::rc::Rc;

pub fn run(
    _this: &JsValue,
    args: &[JsValue],
    _ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    eprintln!("run called");
    Ok(JsValue::undefined())
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    let menu = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_copy_closure({
            let app = Rc::downgrade(&app).as_ptr();
            move |_this, args, ctx| run(_this, args, ctx, unsafe { &mut (*app).borrow_mut() })
        }),
    )
    .name(js_string!("run"))
    .build();

    Ok((
        js_string!("core"),
        Module::synthetic(
            // Make sure to list all exports beforehand.
            &[js_string!("run")],
            // The initializer is evaluated every time a module imports this synthetic module,
            // so we avoid creating duplicate objects by capturing and cloning them instead.
            boa_engine::module::SyntheticModuleInitializer::from_copy_closure_with_captures(
                |module, fns, _| {
                    module.set_export(&js_string!("run"), fns.0.clone().into())?;

                    Ok(())
                },
                (menu,),
            ),
            None,
            context,
        ),
    ))
}

use std::string::String as StdString;

use anyhow::Result;
use rquickjs::{function::Rest, Coerced, Context, Ctx, Function, Object, Runtime};

use crate::{modules, platform};

pub fn run(js_code: &str) -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        register_ahk(&ctx)?;
        ctx.eval::<(), _>(js_code)?;
        Ok(())
    })
}

fn register_ahk(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    let ahk = Object::new(ctx.clone())?;

    ahk.set("platform", build_platform(ctx)?)?;

    for module in modules::all() {
        ahk.set(module.name, build_module(ctx, module.methods)?)?;
    }

    globals.set("ahk", ahk)?;
    Ok(())
}

fn build_platform<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let platform_object = Object::new(ctx.clone())?;
    platform_object.set("name", platform::platform_name())?;
    platform_object.set(
        "init",
        Function::new(ctx.clone(), || {
            platform::init_log();
        })?,
    )?;
    Ok(platform_object)
}

fn build_module<'js>(
    ctx: &Ctx<'js>,
    methods: &'static [modules::ModuleMethod],
) -> rquickjs::Result<Object<'js>> {
    let module_object = Object::new(ctx.clone())?;

    for &(name, function) in methods {
        module_object.set(
            name,
            Function::new(
                ctx.clone(),
                move |args: Rest<Coerced<StdString>>| -> StdString {
                    let owned_args = args.0.into_iter().map(|arg| arg.0).collect::<Vec<_>>();
                    let refs = owned_args
                        .iter()
                        .map(|arg| arg.as_str())
                        .collect::<Vec<_>>();
                    function(&refs)
                },
            )?,
        )?;
    }

    Ok(module_object)
}

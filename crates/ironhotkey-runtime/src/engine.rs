use anyhow::Result;
use rquickjs::{
    function::{Opt, Rest},
    Coerced, Context, Ctx, Function, Object, Runtime,
};

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
    ahk.set("maths", build_maths(ctx)?)?;

    for module in modules::all() {
        if module.name == "maths" {
            continue;
        }
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

fn build_maths<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let module_object = Object::new(ctx.clone())?;

    module_object.set(
        "Abs",
        Function::new(ctx.clone(), |number: f64| modules::maths::abs(number))?,
    )?;
    module_object.set(
        "ACos",
        Function::new(ctx.clone(), |number: f64| modules::maths::a_cos(number))?,
    )?;
    module_object.set(
        "ASin",
        Function::new(ctx.clone(), |number: f64| modules::maths::a_sin(number))?,
    )?;
    module_object.set(
        "Asc",
        Function::new(ctx.clone(), |text: String| modules::maths::asc(&text))?,
    )?;
    module_object.set(
        "ATan",
        Function::new(ctx.clone(), |number: f64| modules::maths::a_tan(number))?,
    )?;
    module_object.set(
        "Ceil",
        Function::new(ctx.clone(), |number: f64| modules::maths::ceil(number))?,
    )?;
    module_object.set(
        "Chr",
        Function::new(ctx.clone(), |code: u32| modules::maths::chr(code))?,
    )?;
    module_object.set(
        "Cos",
        Function::new(ctx.clone(), |number: f64| modules::maths::cos(number))?,
    )?;
    module_object.set(
        "Exp",
        Function::new(ctx.clone(), |number: f64| modules::maths::exp(number))?,
    )?;
    module_object.set(
        "Floor",
        Function::new(ctx.clone(), |number: f64| modules::maths::floor(number))?,
    )?;
    module_object.set(
        "Format",
        Function::new(ctx.clone(), |format_str: String, values: Rest<String>| {
            let refs = values.0.iter().map(String::as_str).collect::<Vec<_>>();
            modules::maths::format(&format_str, &refs)
        })?,
    )?;
    module_object.set(
        "FormatTime",
        Function::new(
            ctx.clone(),
            |timestamp: Opt<String>, pattern: Opt<String>| {
                modules::maths::format_time(timestamp.0.as_deref(), pattern.0.as_deref())
            },
        )?,
    )?;
    module_object.set(
        "Ln",
        Function::new(ctx.clone(), |number: f64| modules::maths::ln(number))?,
    )?;
    module_object.set(
        "Log",
        Function::new(ctx.clone(), |number: f64| modules::maths::log(number))?,
    )?;
    module_object.set(
        "Math",
        Function::new(ctx.clone(), |expression: String| {
            modules::maths::math(&expression)
        })?,
    )?;
    module_object.set(
        "Max",
        Function::new(ctx.clone(), |numbers: Rest<f64>| {
            modules::maths::max(&numbers.0)
        })?,
    )?;
    module_object.set(
        "Min",
        Function::new(ctx.clone(), |numbers: Rest<f64>| {
            modules::maths::min(&numbers.0)
        })?,
    )?;
    module_object.set(
        "Mod",
        Function::new(ctx.clone(), |dividend: f64, divisor: f64| {
            modules::maths::mod_fn(dividend, divisor)
        })?,
    )?;
    module_object.set(
        "NumGet",
        Function::new(
            ctx.clone(),
            |var_or_address: String, offset: Opt<i32>, kind: Opt<String>| {
                modules::maths::num_get(&var_or_address, offset.0, kind.0.as_deref())
            },
        )?,
    )?;
    module_object.set(
        "NumPut",
        Function::new(
            ctx.clone(),
            |number: f64, var_or_address: String, offset: Opt<i32>, kind: Opt<String>| {
                modules::maths::num_put(number, &var_or_address, offset.0, kind.0.as_deref())
            },
        )?,
    )?;
    module_object.set(
        "Ord",
        Function::new(ctx.clone(), |text: String| modules::maths::ord(&text))?,
    )?;
    module_object.set(
        "Random",
        Function::new(ctx.clone(), |min: Opt<f64>, max: Opt<f64>| {
            modules::maths::random(min.0, max.0)
        })?,
    )?;
    module_object.set(
        "Round",
        Function::new(ctx.clone(), |number: f64, digits: Opt<i32>| {
            modules::maths::round(number, digits.0)
        })?,
    )?;
    module_object.set(
        "Sin",
        Function::new(ctx.clone(), |number: f64| modules::maths::sin(number))?,
    )?;
    module_object.set(
        "Sqrt",
        Function::new(ctx.clone(), |number: f64| modules::maths::sqrt(number))?,
    )?;
    module_object.set(
        "Tan",
        Function::new(ctx.clone(), |number: f64| modules::maths::tan(number))?,
    )?;

    Ok(module_object)
}

fn build_module<'js>(
    ctx: &Ctx<'js>,
    methods: &'static [modules::ModuleMethod],
) -> rquickjs::Result<Object<'js>> {
    let module_object = Object::new(ctx.clone())?;

    for &(name, function) in methods {
        module_object.set(
            name,
            Function::new(ctx.clone(), move |args: Rest<Coerced<String>>| -> String {
                let owned_args = args.0.into_iter().map(|arg| arg.0).collect::<Vec<_>>();
                let refs = owned_args
                    .iter()
                    .map(|arg| arg.as_str())
                    .collect::<Vec<_>>();
                function(&refs)
            })?,
        )?;
    }

    Ok(module_object)
}

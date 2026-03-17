pub mod directives;
pub mod disk;
pub mod env;
pub mod ext;
pub mod flow;
pub mod gui;
pub mod maths;
pub mod misc;
pub mod mnk;
pub mod process;
pub mod registry;
pub mod screen;
pub mod sound;
pub mod string;
pub mod types;
pub mod window;

pub type ModuleMethod = (&'static str, fn(&[&str]) -> String);

pub fn stub_log(module: &str, method: &str, args: &[&str]) -> String {
    println!("[ironhotkey::{module}] {method}({})", args.join(", "));
    String::new()
}

pub struct ModuleDef {
    pub name: &'static str,
    pub methods: &'static [ModuleMethod],
}

pub fn all() -> Vec<ModuleDef> {
    vec![
        ModuleDef {
            name: "env",
            methods: env::METHODS,
        },
        ModuleDef {
            name: "ext",
            methods: ext::METHODS,
        },
        ModuleDef {
            name: "disk",
            methods: disk::METHODS,
        },
        ModuleDef {
            name: "flow",
            methods: flow::METHODS,
        },
        ModuleDef {
            name: "gui",
            methods: gui::METHODS,
        },
        ModuleDef {
            name: "maths",
            methods: maths::METHODS,
        },
        ModuleDef {
            name: "mnk",
            methods: mnk::METHODS,
        },
        ModuleDef {
            name: "misc",
            methods: misc::METHODS,
        },
        ModuleDef {
            name: "types",
            methods: types::METHODS,
        },
        ModuleDef {
            name: "process",
            methods: process::METHODS,
        },
        ModuleDef {
            name: "registry",
            methods: registry::METHODS,
        },
        ModuleDef {
            name: "screen",
            methods: screen::METHODS,
        },
        ModuleDef {
            name: "sound",
            methods: sound::METHODS,
        },
        ModuleDef {
            name: "string",
            methods: string::METHODS,
        },
        ModuleDef {
            name: "window",
            methods: window::METHODS,
        },
        ModuleDef {
            name: "directives",
            methods: directives::METHODS,
        },
    ]
}

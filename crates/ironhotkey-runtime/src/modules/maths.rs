use super::{stub_log, ModuleMethod};

pub fn abs(args: &[&str]) -> String {
    stub_log("maths", "Abs", args)
}
pub fn asc(args: &[&str]) -> String {
    stub_log("maths", "Asc", args)
}
pub fn chr(args: &[&str]) -> String {
    stub_log("maths", "Chr", args)
}
pub fn format(args: &[&str]) -> String {
    stub_log("maths", "Format", args)
}
pub fn format_time(args: &[&str]) -> String {
    stub_log("maths", "FormatTime", args)
}
pub fn math(args: &[&str]) -> String {
    stub_log("maths", "Math", args)
}
pub fn num_get(args: &[&str]) -> String {
    stub_log("maths", "NumGet", args)
}
pub fn num_put(args: &[&str]) -> String {
    stub_log("maths", "NumPut", args)
}
pub fn ord(args: &[&str]) -> String {
    stub_log("maths", "Ord", args)
}
pub fn random(args: &[&str]) -> String {
    stub_log("maths", "Random", args)
}
pub fn ceil(args: &[&str]) -> String {
    stub_log("maths", "Ceil", args)
}
pub fn floor(args: &[&str]) -> String {
    stub_log("maths", "Floor", args)
}
pub fn round(args: &[&str]) -> String {
    stub_log("maths", "Round", args)
}
pub fn sqrt(args: &[&str]) -> String {
    stub_log("maths", "Sqrt", args)
}
pub fn sin(args: &[&str]) -> String {
    stub_log("maths", "Sin", args)
}
pub fn cos(args: &[&str]) -> String {
    stub_log("maths", "Cos", args)
}
pub fn tan(args: &[&str]) -> String {
    stub_log("maths", "Tan", args)
}
pub fn a_tan(args: &[&str]) -> String {
    stub_log("maths", "ATan", args)
}
pub fn exp(args: &[&str]) -> String {
    stub_log("maths", "Exp", args)
}
pub fn log(args: &[&str]) -> String {
    stub_log("maths", "Log", args)
}
pub fn ln(args: &[&str]) -> String {
    stub_log("maths", "Ln", args)
}
pub fn mod_fn(args: &[&str]) -> String {
    stub_log("maths", "Mod", args)
}
pub fn max(args: &[&str]) -> String {
    stub_log("maths", "Max", args)
}
pub fn min(args: &[&str]) -> String {
    stub_log("maths", "Min", args)
}
pub fn a_sin(args: &[&str]) -> String {
    stub_log("maths", "ASin", args)
}
pub fn a_cos(args: &[&str]) -> String {
    stub_log("maths", "ACos", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Abs", abs),
    ("ACos", a_cos),
    ("ASin", a_sin),
    ("Asc", asc),
    ("ATan", a_tan),
    ("Ceil", ceil),
    ("Chr", chr),
    ("Cos", cos),
    ("Exp", exp),
    ("Floor", floor),
    ("Format", format),
    ("FormatTime", format_time),
    ("Ln", ln),
    ("Log", log),
    ("Math", math),
    ("Max", max),
    ("Min", min),
    ("Mod", mod_fn),
    ("NumGet", num_get),
    ("NumPut", num_put),
    ("Ord", ord),
    ("Random", random),
    ("Round", round),
    ("Sin", sin),
    ("Sqrt", sqrt),
    ("Tan", tan),
];

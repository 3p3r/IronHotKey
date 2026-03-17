use std::collections::HashMap;
use std::sync::LazyLock;

use ironhotkey_runtime::modules;

static ROUTE_MAP: LazyLock<HashMap<String, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for module in modules::all() {
        for &(method_name, _) in module.methods {
            map.insert(method_name.to_ascii_lowercase(), (module.name, method_name));
        }
    }
    map
});

pub fn route(name: &str) -> (&'static str, String) {
    let key = name.trim().to_ascii_lowercase();
    match ROUTE_MAP.get(&key) {
        Some(&(module, method)) => (module, method.to_string()),
        None => ("misc", sanitize_name(name.trim())),
    }
}

pub fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect();
    if sanitized.is_empty() {
        "command".to_string()
    } else {
        sanitized
    }
}

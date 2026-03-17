use super::{stub_log, ModuleMethod};

pub fn enumerator(args: &[&str]) -> String {
    stub_log("types", "Enumerator", args)
}
pub fn func(args: &[&str]) -> String {
    stub_log("types", "Func", args)
}
pub fn is_func(args: &[&str]) -> String {
    stub_log("types", "IsFunc", args)
}
pub fn is_label(args: &[&str]) -> String {
    stub_log("types", "IsLabel", args)
}
pub fn is_object(args: &[&str]) -> String {
    stub_log("types", "IsObject", args)
}
pub fn is_set(args: &[&str]) -> String {
    stub_log("types", "IsSet", args)
}
pub fn is_by_ref(args: &[&str]) -> String {
    stub_log("types", "IsByRef", args)
}
pub fn object(args: &[&str]) -> String {
    stub_log("types", "Object", args)
}
pub fn obj_add_ref(args: &[&str]) -> String {
    stub_log("types", "ObjAddRef", args)
}
pub fn obj_bind_method(args: &[&str]) -> String {
    stub_log("types", "ObjBindMethod", args)
}
pub fn address_of(args: &[&str]) -> String {
    stub_log("types", "addressOf", args)
}
pub fn deref(args: &[&str]) -> String {
    stub_log("types", "deref", args)
}
pub fn register_class(args: &[&str]) -> String {
    stub_log("types", "registerClass", args)
}
pub fn obj_release(args: &[&str]) -> String {
    stub_log("types", "ObjRelease", args)
}
pub fn obj_raw_get(args: &[&str]) -> String {
    stub_log("types", "ObjRawGet", args)
}
pub fn obj_raw_set(args: &[&str]) -> String {
    stub_log("types", "ObjRawSet", args)
}
pub fn obj_get_base(args: &[&str]) -> String {
    stub_log("types", "ObjGetBase", args)
}
pub fn obj_set_base(args: &[&str]) -> String {
    stub_log("types", "ObjSetBase", args)
}
pub fn exception(args: &[&str]) -> String {
    stub_log("types", "Exception", args)
}
pub fn array(args: &[&str]) -> String {
    stub_log("types", "Array", args)
}
pub fn obj_clone(args: &[&str]) -> String {
    stub_log("types", "ObjClone", args)
}
pub fn obj_count(args: &[&str]) -> String {
    stub_log("types", "ObjCount", args)
}
pub fn obj_delete(args: &[&str]) -> String {
    stub_log("types", "ObjDelete", args)
}
pub fn obj_get_address(args: &[&str]) -> String {
    stub_log("types", "ObjGetAddress", args)
}
pub fn obj_get_capacity(args: &[&str]) -> String {
    stub_log("types", "ObjGetCapacity", args)
}
pub fn obj_has_key(args: &[&str]) -> String {
    stub_log("types", "ObjHasKey", args)
}
pub fn obj_insert(args: &[&str]) -> String {
    stub_log("types", "ObjInsert", args)
}
pub fn obj_insert_at(args: &[&str]) -> String {
    stub_log("types", "ObjInsertAt", args)
}
pub fn obj_length(args: &[&str]) -> String {
    stub_log("types", "ObjLength", args)
}
pub fn obj_max_index(args: &[&str]) -> String {
    stub_log("types", "ObjMaxIndex", args)
}
pub fn obj_min_index(args: &[&str]) -> String {
    stub_log("types", "ObjMinIndex", args)
}
pub fn obj_new_enum(args: &[&str]) -> String {
    stub_log("types", "ObjNewEnum", args)
}
pub fn obj_pop(args: &[&str]) -> String {
    stub_log("types", "ObjPop", args)
}
pub fn obj_push(args: &[&str]) -> String {
    stub_log("types", "ObjPush", args)
}
pub fn obj_remove(args: &[&str]) -> String {
    stub_log("types", "ObjRemove", args)
}
pub fn obj_remove_at(args: &[&str]) -> String {
    stub_log("types", "ObjRemoveAt", args)
}
pub fn obj_set_capacity(args: &[&str]) -> String {
    stub_log("types", "ObjSetCapacity", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Array", array),
    ("Enumerator", enumerator),
    ("Exception", exception),
    ("Func", func),
    ("IsByRef", is_by_ref),
    ("IsFunc", is_func),
    ("IsLabel", is_label),
    ("IsObject", is_object),
    ("IsSet", is_set),
    ("Object", object),
    ("ObjAddRef", obj_add_ref),
    ("ObjBindMethod", obj_bind_method),
    ("ObjClone", obj_clone),
    ("ObjCount", obj_count),
    ("ObjDelete", obj_delete),
    ("ObjGetAddress", obj_get_address),
    ("ObjGetBase", obj_get_base),
    ("ObjGetCapacity", obj_get_capacity),
    ("ObjHasKey", obj_has_key),
    ("ObjInsert", obj_insert),
    ("ObjInsertAt", obj_insert_at),
    ("ObjLength", obj_length),
    ("ObjMaxIndex", obj_max_index),
    ("ObjMinIndex", obj_min_index),
    ("ObjNewEnum", obj_new_enum),
    ("ObjPop", obj_pop),
    ("ObjPush", obj_push),
    ("ObjRawGet", obj_raw_get),
    ("ObjRawSet", obj_raw_set),
    ("ObjRemove", obj_remove),
    ("ObjRemoveAt", obj_remove_at),
    ("ObjRelease", obj_release),
    ("ObjSetCapacity", obj_set_capacity),
    ("ObjSetBase", obj_set_base),
    ("addressOf", address_of),
    ("deref", deref),
    ("registerClass", register_class),
];

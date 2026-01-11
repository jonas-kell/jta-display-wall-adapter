pub use rust_to_ts_types_derive::TypescriptSerializable;

pub trait TypescriptSerializable {
    fn serialize_to_type(&self) -> String;
}

impl TypescriptSerializable for String {
    fn serialize_to_type(&self) -> String {
        return "string".into();
    }
}

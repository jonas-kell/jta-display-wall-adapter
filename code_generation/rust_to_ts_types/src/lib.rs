pub use rust_to_ts_types_derive::TypescriptSerializable;

pub trait TypescriptSerializable {
    fn serialize_to_type() -> String;
}

impl<T> TypescriptSerializable for Option<T>
where
    T: TypescriptSerializable,
{
    fn serialize_to_type() -> String {
        return format!("{} | null", T::serialize_to_type());
    }
}

impl TypescriptSerializable for String {
    fn serialize_to_type() -> String {
        return "string".into();
    }
}

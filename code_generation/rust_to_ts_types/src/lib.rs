pub use rust_to_ts_types_derive::TypescriptSerializable;

pub trait TypescriptSerializable {
    fn type_name() -> String;

    fn serialize_to_type() -> String;

    fn all_types_output() -> Vec<String>;
}

impl<T> TypescriptSerializable for Option<T>
where
    T: TypescriptSerializable,
{
    fn type_name() -> String {
        return format!("{} | null", T::type_name());
    }

    fn serialize_to_type() -> String {
        return format!("{} | null", T::serialize_to_type());
    }

    fn all_types_output() -> Vec<String> {
        T::all_types_output()
    }
}

impl TypescriptSerializable for String {
    fn type_name() -> String {
        return "string".into();
    }

    fn serialize_to_type() -> String {
        return "string".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

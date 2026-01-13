use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
pub use rust_to_ts_types_derive::TypescriptSerializable;
use uuid::Uuid;

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
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!("{} | null", T::type_name());
    }

    fn all_types_output() -> Vec<String> {
        T::all_types_output()
    }
}

impl<T> TypescriptSerializable for Vec<T>
where
    T: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!("{}[]", T::type_name());
    }

    fn all_types_output() -> Vec<String> {
        T::all_types_output()
    }
}

pub trait NumberMarker {}
impl NumberMarker for u8 {}
impl NumberMarker for u16 {}
impl NumberMarker for u32 {}
impl NumberMarker for u64 {}
impl NumberMarker for u128 {}
impl NumberMarker for i8 {}
impl NumberMarker for i16 {}
impl NumberMarker for i32 {}
impl NumberMarker for i64 {}
impl NumberMarker for i128 {}
impl NumberMarker for f32 {}
impl NumberMarker for f64 {}
impl NumberMarker for isize {}
impl NumberMarker for usize {}

impl<U, T> TypescriptSerializable for HashMap<U, T>
where
    U: NumberMarker,
    T: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!("{{ [key: number]: {} }}", T::type_name());
    }

    fn all_types_output() -> Vec<String> {
        T::all_types_output()
    }
}

impl<T> TypescriptSerializable for HashMap<String, T>
where
    T: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!("{{ [key: string]: {} }}", T::type_name());
    }

    fn all_types_output() -> Vec<String> {
        T::all_types_output()
    }
}

impl<A, B> TypescriptSerializable for (A, B)
where
    A: TypescriptSerializable,
    B: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!("[{}, {}]", A::type_name(), B::type_name(),);
    }

    fn all_types_output() -> Vec<String> {
        let mut collector = Vec::new();

        collector.append(&mut A::all_types_output());
        collector.append(&mut B::all_types_output());

        return collector;
    }
}

impl<A, B, C> TypescriptSerializable for (A, B, C)
where
    A: TypescriptSerializable,
    B: TypescriptSerializable,
    C: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!(
            "[{}, {}, {}]",
            A::type_name(),
            B::type_name(),
            C::type_name()
        );
    }

    fn all_types_output() -> Vec<String> {
        let mut collector = Vec::new();

        collector.append(&mut A::all_types_output());
        collector.append(&mut B::all_types_output());
        collector.append(&mut C::all_types_output());

        return collector;
    }
}

impl<A, B, C, D> TypescriptSerializable for (A, B, C, D)
where
    A: TypescriptSerializable,
    B: TypescriptSerializable,
    C: TypescriptSerializable,
    D: TypescriptSerializable,
{
    fn type_name() -> String {
        Self::serialize_to_type()
    }

    fn serialize_to_type() -> String {
        return format!(
            "[{}, {}, {}, {}]",
            A::type_name(),
            B::type_name(),
            C::type_name(),
            D::type_name()
        );
    }

    fn all_types_output() -> Vec<String> {
        let mut collector = Vec::new();

        collector.append(&mut A::all_types_output());
        collector.append(&mut B::all_types_output());
        collector.append(&mut C::all_types_output());
        collector.append(&mut D::all_types_output());

        return collector;
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

impl TypescriptSerializable for bool {
    fn type_name() -> String {
        return "boolean".into();
    }

    fn serialize_to_type() -> String {
        return "boolean".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for u8 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for i8 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for u16 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for i16 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for u32 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for i32 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for u64 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for i64 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for u128 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for i128 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

// impl TypescriptSerializable for f16 {
//     fn type_name() -> String {
//         return "number".into();
//     }

//     fn serialize_to_type() -> String {
//         return "number".into();
//     }

//     fn all_types_output() -> Vec<String> {
//         Vec::new()
//     }
// }

impl TypescriptSerializable for f32 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for f64 {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

// impl TypescriptSerializable for f128 {
//     fn type_name() -> String {
//         return "number".into();
//     }

//     fn serialize_to_type() -> String {
//         return "number".into();
//     }

//     fn all_types_output() -> Vec<String> {
//         Vec::new()
//     }
// }

impl TypescriptSerializable for usize {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

impl TypescriptSerializable for isize {
    fn type_name() -> String {
        return "number".into();
    }

    fn serialize_to_type() -> String {
        return "number".into();
    }

    fn all_types_output() -> Vec<String> {
        Vec::new()
    }
}

fn alias_vec(name: &str) -> Vec<String> {
    [format!("export type {} = string;\n", name)].into()
}

impl TypescriptSerializable for NaiveDate {
    fn type_name() -> String {
        return "NaiveDate".into();
    }

    fn serialize_to_type() -> String {
        return "NaiveDate".into();
    }

    fn all_types_output() -> Vec<String> {
        alias_vec("NaiveDate")
    }
}

impl TypescriptSerializable for NaiveDateTime {
    fn type_name() -> String {
        return "NaiveDateTime".into();
    }

    fn serialize_to_type() -> String {
        return "NaiveDateTime".into();
    }

    fn all_types_output() -> Vec<String> {
        alias_vec("NaiveDateTime")
    }
}

impl TypescriptSerializable for NaiveTime {
    fn type_name() -> String {
        return "NaiveTime".into();
    }

    fn serialize_to_type() -> String {
        return "NaiveTime".into();
    }

    fn all_types_output() -> Vec<String> {
        alias_vec("NaiveTime")
    }
}

impl TypescriptSerializable for Uuid {
    fn type_name() -> String {
        return "Uuid".into();
    }

    fn serialize_to_type() -> String {
        return "Uuid".into();
    }

    fn all_types_output() -> Vec<String> {
        alias_vec("Uuid")
    }
}

use typescript_type_def::type_expr as ts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeInfo {
    Void,
    Bool,
    Number(Number),
    String,
    Optional(Box<TypeInfo>),
    Array(Box<TypeInfo>),
    Map(Box<TypeInfo>),
    Tuple {
        name: Option<String>,
        elements: Vec<TypeInfo>,
    },
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    TaggedEnum {
        name: String,
        variants: Vec<Variant>,
    },
    StringEnum {
        name: String,
        variants: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Variant {
    pub(crate) name: String,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Number {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) ty: TypeInfo,
}

impl From<&ts::TypeInfo> for TypeInfo {
    fn from(info: &ts::TypeInfo) -> Self {
        match info {
            ts::TypeInfo::Native(n) => ctype_from_expr(&n.r#ref, None),
            ts::TypeInfo::Defined(d) => {
                let name = d.def.name.0.to_string();
                match d.def.def {
                    ts::TypeExpr::Name(ts::TypeName {
                        name: ts::Ident("number"),
                        ..
                    }) => match name.as_str() {
                        "U8" => Self::Number(Number::U8),
                        "U16" => Self::Number(Number::U16),
                        "U32" => Self::Number(Number::U32),
                        "U64" | "Usize" => Self::Number(Number::U64),
                        "I8" => Self::Number(Number::I8),
                        "I16" => Self::Number(Number::I16),
                        "I32" => Self::Number(Number::I32),
                        "I64" | "ISize" => Self::Number(Number::I64),
                        "F32" => Self::Number(Number::F32),
                        "F64" => Self::Number(Number::F64),
                        n => panic!("Could not handle `{n}` in {info:?}"),
                    },
                    ts::TypeExpr::Object(o) => TypeInfo::Struct {
                        name,
                        fields: generate_struct_fields(&o),
                    },
                    ts::TypeExpr::Union(u) => {
                        if let Some(variants) = get_string_enum_variants(&u) {
                            TypeInfo::StringEnum { name, variants }
                        } else if let Some(variants) = parse_internally_tagged_union(&u) {
                            TypeInfo::TaggedEnum { name, variants }
                        } else {
                            panic!("Could not convert {u:?}",);
                        }
                    }
                    e => ctype_from_expr(&e, Some(name)),
                }
            }
        }
    }
}

fn parse_internally_tagged_union(u: &ts::TypeUnion) -> Option<Vec<Variant>> {
    if u.members.is_empty() {
        return None;
    }
    let mut tag_field = None;
    let mut variants = vec![];
    for m in u.members {
        match m {
            ts::TypeExpr::Object(obj) if obj.fields.len() == 1 => {
                let f = &obj.fields[0];
                if let ts::TypeExpr::String(s) = &f.r#type {
                    let tf = f.name.value.to_string();
                    if tag_field.get_or_insert_with(|| tf.clone()) != &tf {
                        return None;
                    }
                    variants.push(Variant {
                        name: s.value.to_string(),
                        fields: vec![],
                    });
                } else {
                    return None;
                }
            }
            ts::TypeExpr::Intersection(i) if i.members.len() == 2 => {
                let tag_obj = match &i.members[0] {
                    ts::TypeExpr::Object(a) => a,
                    _ => return None,
                };
                let data_obj = resolve_to_object(&i.members[1])?;
                if tag_obj.fields.len() != 1 {
                    return None;
                }
                let f = &tag_obj.fields[0];
                if let ts::TypeExpr::String(s) = &f.r#type {
                    let tf = f.name.value.to_string();
                    if tag_field.get_or_insert_with(|| tf.clone()) != &tf {
                        return None;
                    }
                    let fields = generate_struct_fields(&data_obj);
                    variants.push(Variant {
                        name: s.value.to_string(),
                        fields,
                    });
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    assert_eq!(tag_field?, "kind");
    Some(variants)
}
pub(crate) fn resolve_to_object(expr: &ts::TypeExpr) -> Option<ts::TypeObject> {
    match expr {
        ts::TypeExpr::Object(o) => Some(*o),
        ts::TypeExpr::Ref(ts::TypeInfo::Defined(d)) => resolve_to_object(&d.def.def),
        _ => None,
    }
}
pub(crate) fn get_string_enum_variants(u: &ts::TypeUnion) -> Option<Vec<String>> {
    if !is_string_enum(u) {
        return None;
    }
    Some(
        u.members
            .iter()
            .filter_map(|m| match m {
                ts::TypeExpr::String(s) => Some(s.value.to_string()),
                _ => None,
            })
            .collect(),
    )
}

fn generate_struct_fields(obj: &ts::TypeObject) -> Vec<Field> {
    obj.fields
        .iter()
        .map(|f| {
            let name = f.name.value.to_string();
            let mut ty = ctype_from_expr(&f.r#type, None);
            if f.optional && !matches!(ty, TypeInfo::Optional(_)) {
                ty = TypeInfo::Optional(Box::new(ty));
            }
            Field { name, ty }
        })
        .collect()
}

pub(crate) fn is_string_enum(u: &ts::TypeUnion) -> bool {
    !u.members.is_empty()
        && u.members
            .iter()
            .all(|m| matches!(m, ts::TypeExpr::String(_)))
}

fn ctype_from_expr(expr: &ts::TypeExpr, name: Option<String>) -> TypeInfo {
    match expr {
        ts::TypeExpr::Ref(info) => TypeInfo::from(*info),
        ts::TypeExpr::Name(n) => match n.name.0 {
            "number" => TypeInfo::Number(Number::F64),
            "string" => TypeInfo::String,
            "boolean" => TypeInfo::Bool,
            "null" | "undefined" | "void" | "never" => TypeInfo::Void,
            "Record" if n.generic_args.len() == 2 => {
                if let Some(name) = name {
                    panic!("name {name} found in {expr:?}");
                }
                TypeInfo::Map(Box::new(ctype_from_expr(&n.generic_args[1], None)))
            }
            _ => unimplemented!(), // other => Type::Struct(other.to_string()),
        },
        ts::TypeExpr::Array(a) => {
            if let Some(name) = name {
                panic!("name {name} found in {expr:?}");
            }
            TypeInfo::Array(Box::new(ctype_from_expr(a.item, None)))
        }
        ts::TypeExpr::Union(u) => {
            let non_null: Vec<_> = u
                .members
                .iter()
                .filter(|m| !matches!(m, ts::TypeExpr::Name(n) if n.name.0 == "null"))
                .collect();
            if non_null.len() < u.members.len() && non_null.len() == 1 {
                if let Some(name) = name {
                    panic!("name {name} found in {expr:?}");
                }
                TypeInfo::Optional(Box::new(ctype_from_expr(non_null[0], None)))
            } else if non_null.len() == 1 {
                ctype_from_expr(non_null[0], None)
            } else {
                TypeInfo::Void // TODO: tagged union
            }
        }
        ts::TypeExpr::Tuple(t) if t.elements.is_empty() => TypeInfo::Void,
        ts::TypeExpr::Tuple(t) => TypeInfo::Tuple {
            name,
            elements: t
                .elements
                .iter()
                .map(|e| ctype_from_expr(e, None))
                .collect(),
        },
        ts::TypeExpr::String(_) => TypeInfo::String,
        _ => TypeInfo::Void,
    }
}

#[cfg(test)]
use crate::typescript::TypeDef;

#[test]
fn test_nested_struct() {
    #[derive(TypeDef)]
    struct Foo {
        _bar: Vec<u8>,
        _baz: std::collections::HashMap<String, i8>,
    }
    assert_eq!(
        TypeInfo::from(&Foo::INFO),
        TypeInfo::Struct {
            name: "Foo".to_owned(),
            fields: vec![
                Field {
                    name: "_bar".to_owned(),
                    ty: TypeInfo::Array(Box::new(TypeInfo::Number(Number::U8)))
                },
                Field {
                    name: "_baz".to_owned(),
                    ty: TypeInfo::Map(Box::new(TypeInfo::Number(Number::I8)))
                }
            ]
        }
    );
}

#[test]
fn test_string_enum() {
    #[derive(TypeDef)]
    enum Foo {
        _A,
        _B,
        _C,
    }
    assert_eq!(
        TypeInfo::from(&Foo::INFO),
        TypeInfo::StringEnum {
            name: "Foo".to_owned(),
            variants: vec!["_A".to_owned(), "_B".to_owned(), "_C".to_owned()]
        }
    );
}

#[test]
fn test_tagged_enum() {
    #[derive(TypeDef)]
    #[serde(tag = "kind")]
    enum Foo {
        _A { name: String, count: u32 },
        _B,
    }
    assert_eq!(
        TypeInfo::from(&Foo::INFO),
        TypeInfo::TaggedEnum {
            name: "Foo".to_owned(),
            variants: vec![
                Variant {
                    name: "_A".to_owned(),
                    fields: vec![
                        Field {
                            name: "name".to_owned(),
                            ty: TypeInfo::String
                        },
                        Field {
                            name: "count".to_owned(),
                            ty: TypeInfo::Number(Number::U32)
                        }
                    ]
                },
                Variant {
                    name: "_B".to_owned(),
                    fields: vec![]
                }
            ]
        }
    );
}

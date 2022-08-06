#![allow(dead_code)]
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Module,
    Int,
    Float,
    Bool,
    Char,
    String,
    Timestamp,
    Array(Array),
    Object(Object),
    Union(Vec<Type>),
    Void,
    Custom(String),
    Fn(Function),
    Unknown,
}

impl core::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Module => f.write_str("module")?,
            Type::Int => f.write_str("int")?,
            Type::Float => f.write_str("float")?,
            Type::Bool => f.write_str("bool")?,
            Type::Char => f.write_str("char")?,
            Type::String => f.write_str("string")?,
            Type::Timestamp => f.write_str("timestamp")?,
            Type::Array(_) => f.write_str("array")?,
            Type::Object(obj) => {
                if f.alternate() {
                    if obj.fields.is_empty() {
                        return f.write_str("#{ }");
                    }

                    f.write_str("#{\n")?;

                    for (field_name, field_ty) in &obj.fields {
                        f.write_str("  ")?;
                        f.write_str(field_name)?;
                        f.write_str(": ")?;
                        field_ty.fmt(f)?;
                        f.write_str("\n")?;
                    }

                    f.write_str("}")?;
                } else {
                    f.write_str("object")?;
                }
            }
            Type::Union(_) => f.write_str("union")?,
            Type::Void => f.write_str("void")?,
            Type::Custom(c) => f.write_str(c)?,
            Type::Fn(ty) => {
                if f.alternate() {
                    if ty.is_closure {
                        f.write_str("|")?;
                    } else {
                        f.write_str("(")?;
                    }

                    let mut first = true;
                    for (param_name, param_ty) in &ty.params {
                        if !first {
                            f.write_str(", ")?;
                        }

                        f.write_str(param_name)?;

                        f.write_str(": ")?;

                        param_ty.fmt(f)?;

                        first = false;
                    }

                    if ty.is_closure {
                        f.write_str("|")?;
                    } else {
                        f.write_str(")")?;
                    }

                    f.write_str(" -> ")?;
                    ty.ret.fmt(f)?;
                } else {
                    f.write_str("function")?;
                }
            }
            Type::Unknown => f.write_str("?")?,
        };

        Ok(())
    }
}

impl Type {
    pub(crate) fn dedup(&mut self) {
        if let Type::Union(u) = self {
            u.dedup();
            for sub in u {
                sub.dedup();
            }
        }
    }
}

impl core::ops::Add for Type {
    type Output = Type;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Type::Union(mut u) => {
                if !u.contains(&rhs) {
                    u.push(rhs);
                }
                Type::Union(u)
            }
            t => {
                if t == rhs {
                    t
                } else {
                    Type::Union(vec![t, rhs])
                }
            }
        }
    }
}

impl<'a> core::ops::Add<&'a Type> for Type {
    type Output = Type;

    fn add(self, rhs: &'a Type) -> Self::Output {
        match self {
            Type::Union(mut u) => {
                if !u.contains(rhs) {
                    u.push(rhs.clone());
                }
                Type::Union(u)
            }
            t => {
                if t == *rhs {
                    t
                } else {
                    Type::Union(vec![t, rhs.clone()])
                }
            }
        }
    }
}

impl core::ops::AddAssign for Type {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            Type::Union(u) => {
                if !u.contains(&rhs) {
                    u.push(rhs);
                }
                *self = Type::Union(mem::take(u));
            }
            t => {
                if t != &rhs {
                    let this = mem::take(t);
                    *t = Type::Union(vec![this, rhs]);
                }
            }
        }
    }
}

impl<'a> core::ops::AddAssign<&'a Type> for Type {
    fn add_assign(&mut self, rhs: &'a Type) {
        match self {
            Type::Union(u) => {
                if !u.contains(rhs) {
                    u.push(rhs.clone());
                }
                *self = Type::Union(mem::take(u));
            }
            t => {
                if t != rhs {
                    let this = mem::take(t);
                    *t = Type::Union(vec![this, rhs.clone()]);
                }
            }
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub item_types: Box<Type>,
    pub known_items: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub is_closure: bool,
    pub params: Vec<(String, Type)>,
    pub ret: Box<Type>,
}

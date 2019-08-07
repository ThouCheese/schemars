use crate as schemars;
use crate::{MakeSchema, MakeSchemaError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, MakeSchema)]
#[serde(untagged)]
pub enum Schema {
    Bool(bool),
    Ref(SchemaRef),
    Object(SchemaObject),
}

impl From<SchemaObject> for Schema {
    fn from(o: SchemaObject) -> Self {
        Schema::Object(o)
    }
}

impl From<bool> for Schema {
    fn from(b: bool) -> Self {
        Schema::Bool(b)
    }
}

impl From<SchemaRef> for Schema {
    fn from(r: SchemaRef) -> Self {
        Schema::Ref(r)
    }
}

fn extend<A, E: Extend<A>>(mut a: E, b: impl IntoIterator<Item = A>) -> E {
    a.extend(b);
    a
}

impl Schema {
    pub fn flatten(self, other: Self) -> Result {
        let s1 = self.ensure_flattenable()?;
        let s2 = other.ensure_flattenable()?;
        Ok(Schema::Object(SchemaObject {
            schema: s1.schema.or(s2.schema),
            id: s1.id.or(s2.id),
            title: s1.title.or(s2.title),
            description: s1.description.or(s2.description),
            items: s1.items.or(s2.items),
            properties: extend(s1.properties, s2.properties),
            required: extend(s1.required, s2.required),
            definitions: extend(s1.definitions, s2.definitions),
            extensions: extend(s1.extensions, s2.extensions),
            // TODO do the following make sense?
            instance_type: s1.instance_type.or(s2.instance_type),
            enum_values: s1.enum_values.or(s2.enum_values),
            all_of: s1.all_of.or(s2.all_of),
            any_of: s1.any_of.or(s2.any_of),
            one_of: s1.one_of.or(s2.one_of),
            not: s1.not.or(s2.not),
        }))
    }

    fn ensure_flattenable(self) -> Result<SchemaObject> {
        let s = match self {
            Schema::Object(s) => s,
            s => {
                return Err(MakeSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s,
                ))
            }
        };
        match s.instance_type {
            Some(SingleOrVec::Single(ref t)) if **t != InstanceType::Object => {
                Err(MakeSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s.into(),
                ))
            }
            Some(SingleOrVec::Vec(ref t)) if !t.contains(&InstanceType::Object) => {
                Err(MakeSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s.into(),
                ))
            }
            _ => Ok(s),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, MakeSchema)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, MakeSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SchemaObject {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub instance_type: Option<SingleOrVec<InstanceType>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<SingleOrVec<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub properties: Map<String, Schema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub definitions: Map<String, Schema>,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, MakeSchema)]
#[serde(rename_all = "camelCase")]
pub enum InstanceType {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, MakeSchema)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(Box<T>),
    Vec(Vec<T>),
}

impl<T> From<T> for SingleOrVec<T> {
    fn from(single: T) -> Self {
        SingleOrVec::Single(Box::new(single))
    }
}

impl<T> From<Vec<T>> for SingleOrVec<T> {
    fn from(vec: Vec<T>) -> Self {
        SingleOrVec::Vec(vec)
    }
}

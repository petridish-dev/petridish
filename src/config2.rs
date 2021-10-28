use serde::Deserialize;

// #[derive(Deserialize, Debug, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct PromptConfig {
//     pub prompts: Vec<PromptItem>,
// }

#[derive(Debug, PartialEq)]
pub struct PromptItem {
    name: String,
    type_: i32,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]

pub enum PromptType {
    Int,
    Float,
    Boolean,
    String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum PromptValue {
    Int(i32),
    Float(f32),
    Boolean(bool),
    String(String),
}

// impl PromptValue {
//     pub fn try_convert(&self, type_name: PromptType) -> Result<Self, String> {
//         match self {
//             PromptValue::Int(v) => match type_name {
//                 PromptType::Int => Ok(Self::Int(*v)),
//                 PromptType::Float => Ok(Self::Float(*v as f32)),
//                 PromptType::Boolean => Ok(Self::Boolean(*v != 0)),
//                 PromptType::String => Ok(Self::String(v.to_string())),
//             },
//             PromptValue::Float(v) => match type_name {
//                 PromptType::Int => Ok(Self::Int(*v as i32)),
//                 PromptType::Float => Ok(Self::Float(*v)),
//                 PromptType::Boolean => Ok(Self::Boolean(*v != 0.0)),
//                 PromptType::String => Ok(Self::String(v.to_string())),
//             },
//             PromptValue::Boolean(v) => match type_name {
//                 PromptType::Int => {
//                     if *v {
//                         Ok(Self::Int(1))
//                     } else {
//                         Ok(Self::Int(0))
//                     }
//                 }
//                 PromptType::Float => {
//                     if *v {
//                         Ok(Self::Float(1.0))
//                     } else {
//                         Ok(Self::Float(0.0))
//                     }
//                 }
//                 PromptType::Boolean => Ok(Self::Boolean(*v)),
//                 PromptType::String => Ok(Self::String(v.to_string())),
//             },
//             PromptValue::String(v) => match type_name {
//                 PromptType::Int => Ok(Self::Int(
//                     v.parse::<i32>()
//                         .map_err(|_| "type need `int`".to_string())?,
//                 )),
//                 PromptType::Float => Ok(Self::Float(
//                     v.parse::<f32>()
//                         .map_err(|_| "type need `float`".to_string())?,
//                 )),
//                 PromptType::Boolean => Ok(Self::Boolean(
//                     v.to_lowercase()
//                         .parse::<bool>()
//                         .map_err(|_| "type need `bool`".to_string())?,
//                 )),
//                 PromptType::String => Ok(Self::String(v.to_string())),
//             },
//         }
//     }
// }

#[derive(Debug, PartialEq)]
pub enum PromptKind {
    Normal {
        type_: Option<PromptType>,
        default: Option<PromptValue>,
    },
}

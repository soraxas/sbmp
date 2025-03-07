use core::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParamValue {
    // Define the Param enum
    Bool(bool),
    Real(f64),
    Int(i32),
    String(String),
}

pub type ParamSetter = dyn Fn(ParamValue) -> bool + Send + Sync;
pub type ParamGetter = dyn Fn() -> ParamValue + Send + Sync;

pub struct Param {
    // Define the Param struct
    name: String,
    range_suggestion: String,

    /// for user to set the value
    setter: Box<ParamSetter>,
    /// for user to get the value
    getter: Box<ParamGetter>,
}

/// Implement the Debug trait for Param
impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[derive(Debug)]
        struct MyParam {
            name: String,
            value: ParamValue,
            range_suggestion: String,
        }

        let Self {
            name,
            // self.g,
            range_suggestion,
            ..
        } = self;

        let value = self.get_value();
        // per Chayim Friedman’s suggestion
        fmt::Debug::fmt(
            &MyParam {
                name: name.clone(),
                value: value.clone(),
                range_suggestion: range_suggestion.clone(),
            },
            f,
        )
    }
}

impl Param {
    fn get_name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn get_value(&self) -> ParamValue {
        (self.getter)()
    }
    fn set_value(&mut self, value: ParamValue) -> bool {
        (self.setter)(value)
    }
    fn set_range_suggestion(&mut self, range_suggestion: String) {
        self.range_suggestion = range_suggestion;
    }
    fn get_range_suggestion(&self) -> &str {
        &self.range_suggestion
    }
}

#[derive(Debug, Default)]
pub struct ParamSet {
    // Define the ParamSet struct
    pub params: HashMap<String, Param>,
}

impl ParamSet {
    fn declare_param(&mut self, name: String, setter: Box<ParamSetter>, getter: Box<ParamGetter>) {
        self.params.insert(
            name.clone(),
            Param {
                name,
                range_suggestion: "".to_string(),
                setter,
                getter,
            },
        );
    }
}

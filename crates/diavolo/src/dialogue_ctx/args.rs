use boa_engine::{
    Context, JsResult, JsValue, NativeFunction, js_string,
    property::{Attribute, PropertyDescriptor},
};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Args(serde_json::Value);

impl Deref for Args {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Args {
    pub fn new(args: serde_json::Value) -> Self {
        Self(args)
    }

    pub fn try_to_parsed(
        &self,
        dialogue_args: &dialogue::Args,
    ) -> Result<DialogueArgs, Box<dyn std::error::Error>> {
        let schema = dialogue_args.to_json_schema();
        if let Err(validation_error) = jsonschema::validate(&schema, &self.0) {
            return Err(validation_error.to_owned().into());
        }

        let map = match &self.0 {
            serde_json::Value::Object(map) => map.clone(),
            _ => unreachable!(),
        };

        let mut args = HashMap::<String, ArgVariant>::new();
        map.into_iter().for_each(|(key, value)| {
            use dialogue::ArgType;
            let arg_type = dialogue_args.get(&key.as_str().into()).unwrap();
            let typed: JsValue = match arg_type.type_of() {
                ArgType::Number => value.as_f64().unwrap().into(),
                ArgType::Integer => value.as_i64().unwrap().into(),
                ArgType::Bool => value.as_bool().unwrap().into(),
                ArgType::String => js_string!(value.as_str().unwrap()).into(),
            };
            let arg = if arg_type.is_mutable() {
                ArgVariant::Mutable(MutableVar::new(typed))
            } else {
                ArgVariant::Immutable(typed)
            };
            args.insert(key, arg);
        });

        Ok(DialogueArgs(args))
    }
}

#[derive(Debug)]
pub struct DialogueArgs(HashMap<String, ArgVariant>);

impl Deref for DialogueArgs {
    type Target = HashMap<String, ArgVariant>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum ArgVariant {
    Immutable(JsValue),
    Mutable(MutableVar),
}

impl DialogueArgs {
    pub fn register_in_boa_context(&self, context: &mut Context) -> JsResult<()> {
        self.iter()
            .map(|(key, value)| match value {
                ArgVariant::Immutable(val) => context.register_global_property(
                    js_string!(key.to_owned()),
                    val.clone(),
                    Attribute::ENUMERABLE,
                ),
                ArgVariant::Mutable(mutable_var) => mutable_var.register_as_property(context, key),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct MutableVar {
    value: Arc<Mutex<JsValue>>,
}

impl MutableVar {
    pub fn new(value: JsValue) -> Self {
        Self {
            value: Arc::new(Mutex::new(value)),
        }
    }

    pub fn register_as_property(&self, context: &mut Context, key: &str) -> JsResult<()> {
        let value_for_getter = self.value.clone();
        let getter = unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue,
                      _args: &[JsValue],
                      _context: &mut Context|
                      -> JsResult<JsValue> {
                    let value = value_for_getter.lock().unwrap();
                    Ok(value.clone())
                },
            )
        };

        let value_for_setter = self.value.clone();
        let setter = unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue,
                      args: &[JsValue],
                      _context: &mut Context|
                      -> JsResult<JsValue> {
                    if let Some(new_value) = args.get(0) {
                        let mut value = value_for_setter.lock().unwrap();
                        *value = new_value.clone();
                    }
                    Ok(JsValue::undefined())
                },
            )
        };

        let descriptor = PropertyDescriptor::builder()
            .get(getter.to_js_function(context.realm()))
            .set(setter.to_js_function(context.realm()))
            .enumerable(true)
            .configurable(false)
            .build();

        context
            .global_object()
            .define_property_or_throw(js_string!(key), descriptor, context)?;

        Ok(())
    }
}

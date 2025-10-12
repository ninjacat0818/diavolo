use super::data::Data;
use dialogue::{ChoiceTexts, LangTexts, LineIf, Text, Texts};

use boa_engine::object::IntegrityLevel;
use boa_engine::property::PropertyDescriptor;
use boa_engine::{Context, JsObject, JsResult, JsValue, NativeFunction, Source, js_string};

use std::ops::{Deref, DerefMut, Sub};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct BoaCtx {
    context: Context,
}

impl Deref for BoaCtx {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for BoaCtx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl BoaCtx {
    pub fn eval_if(&mut self, line_if: &LineIf) -> boa_engine::JsResult<bool> {
        let result = self.context.eval(Source::from_bytes(line_if.as_bytes()))?;
        Ok(result
            .as_boolean()
            .expect("line_if should evaluate to a boolean"))
    }

    pub fn eval_str(&mut self, value: impl AsRef<str>) -> boa_engine::JsResult<JsValue> {
        self.context
            .eval(Source::from_bytes(value.as_ref().as_bytes()))
    }

    pub fn eval_texts(&mut self, texts: &Texts) -> JsResult<Texts> {
        match texts {
            Texts::Monolingual(text) => Ok(Texts::Monolingual(self.eval_text(text)?)),
            Texts::Multilingual(lang_texts) => {
                let evaluated_lang_texts = lang_texts
                    .iter()
                    .map(|(language, text)| {
                        Ok::<_, boa_engine::JsError>((language.clone(), self.eval_text(text)?))
                    })
                    .collect::<Result<LangTexts, _>>()?;
                Ok(Texts::Multilingual(evaluated_lang_texts))
            }
        }
    }

    pub fn eval_choice_texts(&mut self, texts: &ChoiceTexts) -> JsResult<ChoiceTexts> {
        texts
            .iter()
            .map(|(key, lang_texts)| {
                let evaluated_lang_texts = self.eval_texts(lang_texts)?;
                Ok((key.clone(), evaluated_lang_texts))
            })
            .collect()
    }

    pub fn eval_text(&mut self, text: &Text) -> JsResult<Text> {
        tracing::debug!("Evaluating text: {}", text);
        let result = self
            .context
            .eval(Source::from_bytes(format!("`{text}`").as_bytes()))?;
        Ok(result
            .as_string()
            .expect("message should evaluate to a string")
            .to_std_string_escaped()
            .into())
    }

    #[cfg(test)]
    pub fn eval_for_assert(&mut self, source: &str) {
        let assert_fn = r#"
            function assert(condition, message) {
                if (!condition) {
                    throw new Error(`assertion failed: ${condition}`);
                }
            }
            function assert_eq(lhs, rhs, message) {
                if (lhs !== rhs) {
                    const msg = [
                        `assertion \`left == right\` failed: ${message}`,
                        `  left: ${lhs}`,
                        ` right: ${rhs}`,
                    ].join('\n');
                    throw new Error(msg);
                }
            }
        "#;
        self.context.eval(Source::from_bytes(assert_fn)).unwrap();
        self.context
            .eval(Source::from_bytes(source))
            .map_err(|e| {
                e.to_opaque(&mut self.context)
                    .to_string(&mut self.context)
                    .unwrap()
                    .to_std_string_escaped()
            })
            .expect("Boa script should run without errors");
    }

    pub fn define_properties(&mut self, data: Arc<Mutex<Data>>) -> JsResult<()> {
        let lines_getter = Self::lines_getter(data.clone());
        let self_getter = Self::line_getter(data.clone(), lines_getter.clone(), 0);
        let prev_getter = Self::line_getter(data.clone(), lines_getter.clone(), -1);
        let next_getter = Self::line_getter(data.clone(), lines_getter.clone(), 1);

        Self::define_property(&mut self.context, "lines", lines_getter)?;
        Self::define_property(&mut self.context, "self", self_getter)?;
        Self::define_property(&mut self.context, "prev", prev_getter)?;
        Self::define_property(&mut self.context, "next", next_getter)?;

        Ok(())
    }

    fn define_property(
        context: &mut Context,
        name: &str,
        getter: NativeFunction,
    ) -> JsResult<bool> {
        context.global_object().define_property_or_throw(
            js_string!(name),
            PropertyDescriptor::builder()
                .get(getter.to_js_function(context.realm()))
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )
    }

    fn line_getter(
        data: Arc<Mutex<Data>>,
        lines_getter: NativeFunction,
        offset: isize,
    ) -> NativeFunction {
        unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue,
                      _args: &[JsValue],
                      context: &mut Context|
                      -> JsResult<JsValue> {
                    let property_name = match offset {
                        -1 => "prev",
                        0 => "self",
                        1 => "next",
                        _ => unreachable!(),
                    };
                    tracing::debug!("Accessing {property_name} property from BoaCtx");
                    let lines = lines_getter.call(&JsValue::undefined(), &[], context)?;
                    let idx = *data.lock().unwrap().state_machine.location().line_position as isize;
                    lines
                        .as_object()
                        .unwrap()
                        .get(idx + offset, context)
                        .or(Ok(JsValue::undefined()))
                },
            )
        }
    }

    fn lines_getter(data: Arc<Mutex<Data>>) -> NativeFunction {
        unsafe {
            NativeFunction::from_closure(
                move |_this: &JsValue,
                      _args: &[JsValue],
                      context: &mut Context|
                      -> JsResult<JsValue> {
                    tracing::debug!("Accessing 'lines' property from BoaCtx");
                    let data = data.lock().unwrap();
                    let lines = data.lines_or_panic();
                    let lines_obj = JsObject::with_object_proto(context.intrinsics());

                    for (index, (line_id, visiting_counting)) in lines.iter().enumerate() {
                        let line_obj = JsObject::with_object_proto(context.intrinsics());

                        let key = js_string!("id");
                        let value = js_string!(line_id.as_ref());
                        line_obj.set(key, value, true, context)?;

                        let key = js_string!("visited");
                        let value = visiting_counting.is_visited();
                        line_obj.set(key, value, true, context)?;

                        let key = js_string!("visited_count");
                        let value = visiting_counting.visited_count();
                        line_obj.set(key, value, true, context)?;

                        let key = js_string!("visited_count_next");
                        let value = visiting_counting.visited_count() + 1;
                        line_obj.set(key, value, true, context)?;

                        use super::visiting_states::VisitingCounting;

                        match visiting_counting {
                            VisitingCounting::Message(_state) => {}
                            VisitingCounting::Confirm(state) => {
                                if let Some(approved) = state.confirmed() {
                                    let key = js_string!("approved");
                                    line_obj.set(key, approved, true, context)?;
                                    let key = js_string!("rejected");
                                    line_obj.set(key, !approved, true, context)?;
                                }
                            }
                            VisitingCounting::Choice(state) => {
                                if let Some(selected) = state.selected() {
                                    let key = js_string!("selected");
                                    let value = js_string!(selected.choice_key.as_str());
                                    line_obj.set(key, value, true, context)?;

                                    let key = js_string!("selected_at");
                                    let value = Self::create_date_from_instant(
                                        &selected.selected_at,
                                        context,
                                    )?;

                                    line_obj.set(key, value, true, context)?;
                                }
                            }
                            VisitingCounting::Eval(_state) => {}
                            VisitingCounting::Goto(_state) => {}
                            VisitingCounting::Call(state) => {
                                if let Some(returned_value) = state.returned_value() {
                                    let key = js_string!("returned");
                                    let value = returned_value.clone();
                                    line_obj.set(key, value, true, context)?;
                                }
                            }
                            VisitingCounting::Return(_state) => {}
                            VisitingCounting::Exit => {}
                        }

                        line_obj.set_integrity_level(IntegrityLevel::Frozen, context)?;

                        lines_obj.set(index, line_obj, true, context)?;
                        lines_obj.define_property_or_throw(
                            js_string!(line_id.as_ref()),
                            PropertyDescriptor::builder()
                                .get(
                                    NativeFunction::from_copy_closure(move |this, _, ctx| {
                                        this.as_object().unwrap().get(index, ctx)
                                    })
                                    .to_js_function(context.realm()),
                                )
                                .enumerable(true)
                                .configurable(false)
                                .build(),
                            context,
                        )?;
                    }

                    lines_obj.set_integrity_level(IntegrityLevel::Frozen, context)?;
                    Ok(lines_obj.into())
                },
            )
        }
    }

    fn create_date_from_instant(
        instant: &std::time::Instant,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .sub(instant.elapsed())
            .as_millis();

        let date_ctor = context
            .global_object()
            .get(js_string!("Date"), context)?
            .as_constructor()
            .unwrap();

        date_ctor.construct(&[JsValue::from(timestamp_ms as f64)], None, context)
    }
}

use boa_engine::JsValue;
use dialogue::{ChoiceTexts, ConfirmResponse, NodeKey, Texts};

pub enum EvaluatedLine {
    Message(Texts),
    Choice(ChoiceTexts, Option<Texts>),
    Confirm(Texts, Option<ConfirmResponse>),
    Eval(JsValue),
    Goto(String),
    Call(NodeKey),
    Return(JsValue),
}

impl EvaluatedLine {
    pub fn into_message_or_panic(self) -> Texts {
        match self {
            EvaluatedLine::Message(texts) => texts,
            _ => panic!("Called into_message on non-Message EvaluatedLine"),
        }
    }

    pub fn into_choice_or_panic(self) -> (ChoiceTexts, Option<Texts>) {
        match self {
            EvaluatedLine::Choice(choice_texts, texts) => (choice_texts, texts),
            _ => panic!("Called into_choice on non-Choice EvaluatedLine"),
        }
    }

    pub fn into_confirm_or_panic(self) -> (Texts, Option<ConfirmResponse>) {
        match self {
            EvaluatedLine::Confirm(texts, response_texts) => (texts, response_texts),
            _ => panic!("Called into_confirm on non-Confirm EvaluatedLine"),
        }
    }

    pub fn into_eval_or_panic(self) -> JsValue {
        match self {
            EvaluatedLine::Eval(value) => value,
            _ => panic!("Called into_eval on non-Eval EvaluatedLine"),
        }
    }

    pub fn into_goto_or_panic(self) -> String {
        match self {
            EvaluatedLine::Goto(line_id_or_index) => line_id_or_index,
            _ => panic!("Called into_goto on non-Goto EvaluatedLine"),
        }
    }

    pub fn into_call_or_panic(self) -> NodeKey {
        match self {
            EvaluatedLine::Call(node_key) => node_key,
            _ => panic!("Called into_call on non-Call EvaluatedLine"),
        }
    }

    pub fn into_return_or_panic(self) -> JsValue {
        match self {
            EvaluatedLine::Return(value) => value,
            _ => panic!("Called into_return on non-Return EvaluatedLine"),
        }
    }
}

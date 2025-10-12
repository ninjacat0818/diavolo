use boa_engine::JsValue;
use dialogue::{ChoiceTexts, NodeKey, Texts};

pub enum EvaluatedLine {
    Message(Texts),
    Choice(ChoiceTexts, Option<Texts>),
    Eval(JsValue),
    Goto(String),
    Call(NodeKey),
    Return(JsValue),
}

impl EvaluatedLine {
    pub fn into_message(self) -> Texts {
        match self {
            EvaluatedLine::Message(texts) => texts,
            _ => panic!("Called into_message on non-Message EvaluatedLine"),
        }
    }

    pub fn into_choice(self) -> (ChoiceTexts, Option<Texts>) {
        match self {
            EvaluatedLine::Choice(choice_texts, texts) => (choice_texts, texts),
            _ => panic!("Called into_choice on non-Choice EvaluatedLine"),
        }
    }

    pub fn into_eval(self) -> JsValue {
        match self {
            EvaluatedLine::Eval(value) => value,
            _ => panic!("Called into_eval on non-Eval EvaluatedLine"),
        }
    }

    pub fn into_goto(self) -> String {
        match self {
            EvaluatedLine::Goto(line_id_or_index) => line_id_or_index,
            _ => panic!("Called into_goto on non-Goto EvaluatedLine"),
        }
    }

    pub fn into_call(self) -> NodeKey {
        match self {
            EvaluatedLine::Call(node_key) => node_key,
            _ => panic!("Called into_call on non-Call EvaluatedLine"),
        }
    }

    pub fn into_return(self) -> JsValue {
        match self {
            EvaluatedLine::Return(value) => value,
            _ => panic!("Called into_return on non-Return EvaluatedLine"),
        }
    }
}

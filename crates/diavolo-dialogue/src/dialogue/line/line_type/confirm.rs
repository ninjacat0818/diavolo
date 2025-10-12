use super::Texts;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Confirm {
    #[serde(rename(serialize = "confirm"))]
    pub texts: Texts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ConfirmOptions>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConfirmOptions {
    response: Option<ConfirmTexts>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct ConfirmTexts {
    yes: Texts,
    no: Texts,
    // more_info: Option<(LangTexts, LangTexts)>,
}

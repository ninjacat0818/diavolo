use super::LangTexts;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Confirm {
    #[serde(flatten)]
    pub texts: LangTexts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ConfirmOptions>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConfirmOptions {
    response: Option<ConfirmTexts>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct ConfirmTexts {
    yes: LangTexts,
    no: LangTexts,
    // more_info: Option<(LangTexts, LangTexts)>,
}

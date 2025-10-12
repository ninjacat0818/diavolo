use color_eyre::eyre::Result;
use diavolo::{Dialogue, DialogueCtx, Runner, Store};
use diavolo_tester::{App, CollectorLayer, LOG_COLLECTOR};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(CollectorLayer::new(LOG_COLLECTOR.clone()))
        .init();

    let engine = diavolo::Engine::default();

    let args = serde_json::json!({
        "x": 42,
        "y": "Hello, Diavolo!"
    });

    let actors = serde_json::json!([
      { "name": "Actor1" },
      { "name": "Actor2" }
    ]);

    let dialogue_ctx: DialogueCtx = DialogueCtx::builder().actors(actors).args(args).build();
    let mut store: Store<'_> = Store::new(&engine, diavolo::Data::with_ctx(dialogue_ctx));
    let dialogue: Dialogue = RAW_DIALOGUE.parse()?;
    let runner: Runner<'_, '_> = Runner::instantiate(&mut store, &dialogue).unwrap();

    App::new(runner).run().await
}

const RAW_DIALOGUE: &str = r#"name: "test dialogue script"
actor:
  num: 2
args:
  x: number
  y: string
nodes:
  main:
  - if: true
    message:
      en: ${lines}
    owner: 0
  - if: false
    message:
      en: ${y}, you have a positive ${x}!
    owner: 0
  - message:
      en: This is a message without condition.
    owner: 0
  - if: false
    message:
      en: This message will not be shown.
    owner: 0
  - message:
      en: Hello from actor 2!
    owner: 1
  - message:
      en: ${lines}
    owner: 1
  - choice:
      - en: Foo text
      - en: Bar text
    options:
      message:
        texts:
          en: This is a choice. Please select.
        owner: 0
"#;

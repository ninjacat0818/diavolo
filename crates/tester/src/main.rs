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

    // let args = serde_json::json!({
    //     "x": 42,
    //     "y": "Hello, Diavolo!"
    // });

    let actors = serde_json::json!([
      // { "name": "Actor1" },
      // { "name": "Actor2" }
    ]);

    let dialogue_ctx: DialogueCtx = DialogueCtx::builder()
        .system_actor(true)
        .actors(actors)
        .expect("Failed to set actors")
        // .args(args)
        .build();
    // let dialogue_ctx: DialogueCtx = DialogueCtx::builder().build();
    let mut store: Store<'_> = Store::new(&engine, diavolo::Data::with_ctx(dialogue_ctx));
    let dialogue: Dialogue = RAW_DIALOGUE.parse()?;
    let runner: Runner<'_, '_> = Runner::instantiate(&mut store, &dialogue).unwrap();

    App::new(runner).run().await
}

const RAW_DIALOGUE: &str = r#"
nodes:
  main:
  - id: q1
    choice:
      tokyo: 東京
      kyoto: 京都
      saitama: さいたま
    options:
      message:
        texts: 日本の首都は？
  - confirm: ファイナルアンサー？
    options:
      response:
        yes: ファイナルアンサー
        no: まだ考えます
  - if: prev.rejected
    goto: q1
  - call: |-
      ${lines.q1.selected === "tokyo" ? "correct" : "incorrect"}
  - if: lines.q1.selected !== "tokyo"
    goto: q1
  - message: 終わり
  - exit: 0
  - message: ここには来ないはず
  correct:
  - message: せいかい！
  incorrect:
  - message: ざんねん！
  - message: ${self.visited_count_next}かいまちがえた
  - message: もう一度挑戦してね！
"#;

// const RAW_DIALOGUE: &str = r#"name: "test dialogue script"
// actor:
//   num: 2
// args:
//   x: number
//   y: string
// nodes:
//   main:
//   - message: こんにちは、世界！ようこそDiavoloへ！
//   - choice:
//       foo:
//         en: Foo text
//       bar:
//         en: Bar text
//     options:
//       message:
//         texts:
//           en: This is a choice. Please select.
//         owner: 0
//   - message:
//       en: You selected ${lines[0].selected}
//   - message:
//       en: x:${x}, y:${y}
//   - message:
//       en: This is a message without condition.
//   - if: false
//     message:
//       en: This message will not be shown.
//   - message:
//       en: Hello from actor 2!
//     owner: 1
// "#;

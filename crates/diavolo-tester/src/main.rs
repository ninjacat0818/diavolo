use color_eyre::eyre::Result;
use diavolo::{Dialogue, Runner, Store};
use diavolo_tester::App;

#[tokio::main]
async fn main() -> Result<()> {
    let engine = diavolo::Engine::default();
    let mut store = Store::new(&engine, diavolo::Data::default());
    let dialogue: Dialogue = RAW_DIALOGUE.parse()?;
    let runner = Runner::instantiate(&mut store, &dialogue);

    App::new(runner).run().await;

    Ok(())
}

const RAW_DIALOGUE: &str = r#"name: "test dialogue script"
actor:
  num: 1
args:
nodes:
  main:
  - if: true
    message:
      en: Hello
    owner: 0
  - message:
      en: How are you?
    owner: 0
  - choice:
      foo:
        en: Foo text
      bar:
        en: Bar text
    options:
      message:
        texts:
          en: This is a choice. Please select.
        owner: 0
"#;

use super::channels::{self, DialogueQueueParams, RunnerOperation};
use super::error::{self, ServerError};
use super::runtimes::Runtimes;

use diavolo::{Dialogue, DialogueCtx, Runner, Store};

use tokio::sync::mpsc;
use tokio::task::LocalSet;

pub async fn run_task(runtimes: Runtimes) -> error::Result<()> {
    let local = LocalSet::new();

    local.spawn_local(async move {
        tracing::debug!("Task started");

        let mut dq_rx = channels::dialogue_queue_rx();
        let mut ro_rx = channels::runner_op_rx();

        while let Some(params) = dq_rx.recv().await {
            tracing::debug!("Received a dialogue queue params");
            if let Err(e) = process_queue(&mut ro_rx, &runtimes, params).await {
                tracing::error!("Error processing dialogue queue: {}", e);
            } else {
                tracing::debug!("Finished processing dialogue queue");
            }
        }
    });

    local
        .run_until(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        })
        .await;

    Ok(())
}

async fn process_queue(
    rx: &mut mpsc::Receiver<RunnerOperation>,
    runtimes: &Runtimes,
    DialogueQueueParams {
        runtime_path,
        actors,
        args,
        dialogue,
    }: DialogueQueueParams,
) -> error::Result<()> {
    tracing::debug!("Processing dialogue for runtime path: {:?}", runtime_path);

    let runtime = runtimes
        .get(&runtime_path)
        .ok_or(ServerError::RuntimeNotFound)?;
    let engine = &runtime.engine;
    let dialogue_ctx = DialogueCtx::builder()
        .system_actor(true)
        .actors(actors)?
        .args(args)
        .build();
    let mut store = Store::new(engine, diavolo::Data::with_ctx(dialogue_ctx));
    let mut runner = Runner::instantiate(&mut store, &dialogue)?;

    loop {
        if runner.is_terminated() {
            tracing::debug!("Dialogue runner terminated");
            break Ok(());
        }

        let Some(op) = rx.recv().await else {
            tracing::warn!("Runner operation channel closed");
            break Ok(());
        };

        tracing::debug!("Received a runner operation: {:?}", op);

        match op {
            RunnerOperation::UpdateView => {
                if let Some(view) = runner.update_view() {
                    tracing::debug!("Updated view: {:?}", view);
                    channels::send_view(view.clone()).await?;
                };
            }
            RunnerOperation::Dispatch(action) => {
                runner.dispatch(action);
            }
        }
    }
}

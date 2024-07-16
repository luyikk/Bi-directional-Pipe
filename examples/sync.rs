use anyhow::{ensure, Result};
use bi_directional_pipe::sync::pipe;
use std::time::Instant;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    let (left, right) = pipe();

    let join: JoinHandle<Result<()>> = tokio::spawn(async move {
        for i in 0..1000000 {
            right.send(i);
            let ok = right.recv().await?;
            ensure!(ok == i + 1, "left return v error!!");
        }
        Ok(())
    });

    let start = Instant::now();
    loop {
        if let Ok(v) = left.recv().await {
            left.send(v + 1);
        } else {
            break;
        }
    }
    println!("time {} sec", start.elapsed().as_secs_f32());

    join.await??;

    Ok(())
}

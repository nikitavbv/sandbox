use {
    std::{sync::Arc, time::{Duration, Instant}},
    tracing::info,
    async_trait::async_trait,
    tokio::{time, sync::Mutex},
    crate::{
        models::io::ModelData,
        scheduling::scheduler::Scheduler,
        context::Context,
    },
};

pub struct AutoShutdownScheduler<T: Scheduler> {
    inner: T,
    activity_at: Arc<Mutex<Instant>>,
}

impl<T: Scheduler> AutoShutdownScheduler<T> {
    pub fn new(inner: T) -> Self {
        let activity_at = Arc::new(Mutex::new(Instant::now()));

        tokio::spawn(monitor_last_activity_at(activity_at.clone()));

        Self {
            inner,
            activity_at,
        }
    }
}

#[async_trait]
impl<T: Scheduler + Send + Sync> Scheduler for AutoShutdownScheduler<T> {
    async fn run(&self, context: Arc<Context>, model_id: &str, input: &ModelData) -> ModelData {
        {
            let mut last_call_time = self.activity_at.lock().await;
            *last_call_time = Instant::now();
        };
        self.inner.run(context, model_id, input).await
    }
}

async fn monitor_last_activity_at(activity_at: Arc<Mutex<Instant>>) {
    info!("auto-shutdown handler started");
    // TODO: add actual auto shutdown here
    /* 
    loop {
        sleep(Duration::from_secs(60)).await;

        let elapsed = {
            let last_call_time = last_call_time.lock().await;
            last_call_time.elapsed()
        };

        if elapsed >= Duration::from_secs(10 * 60) {
            println!("Shutting down the system...");
            // Use the appropriate command for your system
            let _ = Command::new("shutdown").arg("-h").arg("now").status();
            break;
        }
    } 
    */
}
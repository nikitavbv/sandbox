use {
    std::{sync::Arc, time::{Duration, Instant}},
    tracing::info,
    async_trait::async_trait,
    tokio::{time::sleep, sync::Mutex, process::Command},
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
    loop {
        sleep(Duration::from_secs(5)).await;

        let elapsed = {
            let last_call_time = activity_at.lock().await;
            last_call_time.elapsed()
        };

        info!("elapsed: {:?}", elapsed);
        if elapsed >= Duration::from_secs(10) {
            info!("Shutting down the system...");
            let _ = Command::new("sudo").arg("shutdown").arg("-h").arg("now").status();
            break;
        }
    }
}
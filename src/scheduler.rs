use tokio::{sync::mpsc, task};
use url::Url;

pub struct Scheduler {
	queue: mpsc::Sender<FetchTask>,
}

impl Scheduler {
	pub fn setup() -> Self {
		let (tx, mut rx) = mpsc::channel(100);

		task::spawn(async move {
			while let Some(task) = rx.recv().await {
				//
			}
			//
		});

		Self { queue: tx }
	}
}

struct FetchTask {
	url: Url,
}

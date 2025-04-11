use std::collections::VecDeque;
use url::Url;

struct FetchTask {
	url: Url,
}

struct Scheduler {
	queue: VecDeque<FetchTask>,
}

use std::io;

fn main() {
	let mut opml = io::Cursor::new(include_str!("../../subscriptions.opml"));
	let folders_to_import = feedr_core::import::opml_to_feed_folders(&mut opml);

	dbg!(folders_to_import);
}

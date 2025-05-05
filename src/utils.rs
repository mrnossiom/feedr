use std::io::Read;

use eyre::Context;
use opml::OPML;
use url::Url;

#[derive(Debug)]
pub struct ImportedFeed {
	pub title: String,
	pub url: Url,
}

pub fn opml_to_feed_folders<R: Read>(
	mut reader: &mut R,
) -> eyre::Result<impl Iterator<Item = (String, Vec<ImportedFeed>)>> {
	let opml = OPML::from_reader(&mut reader).wrap_err("could not fit feed into model")?;

	let iter = opml.body.outlines.into_iter().map(|folder| {
		let feeds = folder
			.outlines
			.into_iter()
			.map(|outline| ImportedFeed {
				title: outline.title.unwrap(),
				url: outline.xml_url.unwrap().parse().unwrap(),
			})
			.collect::<Vec<_>>();

		(folder.title.unwrap(), feeds)
	});

	Ok(iter)
}

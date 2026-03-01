install-tools:
	cargo install slint-tr-extractor
	cargo install slint-viewer 

extract-string:
	@(find -name \*.slint | xargs slint-tr-extractor -o  track-events_strings.pot)
run_all_examples:
	cargo run --example multiple_incoming_channels && \
    cargo run --example dependency_injection && \
    cargo run --example interval && \
    cargo run --example single_channel_actor

run_tests:
	cargo test

publish: run_tests run_all_examples
	cargo publish -p uactor --allow-dirty

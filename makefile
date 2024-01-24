run_base_example:
	cargo run --example base_sample

run_all_examples:
	cargo run --example base_sample && \
    cargo run --example dependency_injection && \
    cargo run --example interval && \
    cargo run --example single_channel_actor

publish:
	cargo publish -p uactor #--dry-run

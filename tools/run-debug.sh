export RUST_BACKTRACE=1
export RUST_LOG=none,dream_editor=debug,dream_renderer=debug,dream_ecs=debug,dream_math=debug,dream_app=debug,dream_window=debug,dream_fs=debug,dream_resource=debug,dream_tasks=debug,dream_runner=debug,dream_time=debug
cargo run --package dream-runner --bin dream-runner
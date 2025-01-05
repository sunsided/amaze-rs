[private]
help:
    @just --list --unsorted

# Format the code
fmt:
    cargo fmt

# Lint the code
lint:
    cargo clippy --fix --allow-staged

# Render a maze in PPM (portable pixmap) format
test STYLE="double" SEED="17" WIDTH="18" HEIGHT="24":
    cargo run -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style {{ STYLE }}

# Render a maze in PPM (portable pixmap) format
test-ppm SEED="17" WIDTH="18" HEIGHT="24":
    cargo run -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style ppm

# Render a maze in PBM (portable bitmap) format
test-pbm SEED="17" WIDTH="18" HEIGHT="24":
    cargo run -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style pbm

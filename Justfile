[private]
help:
    @just --list --unsorted

# Format the code
fmt:
    cargo fmt

# Lint the code
lint:
    cargo clippy --fix --allow-staged

# Run all tests
test:
    cargo test -p amaze

# Render a maze in PPM (portable pixmap) format
example STYLE="double" SEED="1337" WIDTH="18" HEIGHT="24":
    cargo run --bin amaze-cli -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style {{ STYLE }}

# Render a maze in PPM (portable pixmap) format
example-ppm SEED="1337" WIDTH="18" HEIGHT="24":
    cargo run --bin amaze-cli -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style ppm | tee test.ppm

# Render a maze in PBM (portable bitmap) format
example-pbm SEED="1337" WIDTH="18" HEIGHT="24":
    cargo run --bin amaze-cli -- gen --seed {{ SEED }} --width {{ WIDTH }} --height {{ HEIGHT }} --style pbm | tee test.pbm

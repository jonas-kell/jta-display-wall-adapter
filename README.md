# Display Wall Module by JTA

Software used for running custom display walls.

## Dev

```cmd
docker-compose up
```

## cross compile windows executable on linux

```cmd
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

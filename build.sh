cargo build --target=x86_64-unknown-linux-musl --release

cp ./target/x86_64-unknown-linux-musl/release/server-test .

docker build -t server-test -f Dockerfile.server .

nitro-cli build-enclave --docker-uri server-test --output-file server-test.eif

nitro-cli run-enclave --eif-path server-test.eif --cpu-count 2 --memory 256 --debug-mode

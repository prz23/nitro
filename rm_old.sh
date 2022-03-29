rm -f server-test
rm -f server-test.eif
docker rmi server-test
nitro-cli terminate-enclave --all
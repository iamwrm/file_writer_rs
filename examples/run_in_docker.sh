sudo rm -rf target

docker build -t file-writer-builder -f examples/Dockerfile .

docker run -v $(pwd):/app -w /app file-writer-builder \
    bash -c "bash examples/run.sh"

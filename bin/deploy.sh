#!/usr/bin/env bash

set -euo pipefail

APP_NAME="fm-chain"
TMP_FOLDER="$(mktemp -d)"
FILE_NAME="${APP_NAME}.tar"
IMAGE_FILE="${TMP_FOLDER}/${FILE_NAME}"

trap 'rm -rf "$TMP_FOLDER"' EXIT

docker build --platform=linux/amd64 -t $APP_NAME:latest . -f Dockerfile
docker save $APP_NAME:latest -o $IMAGE_FILE
scp $IMAGE_FILE root@jhx.app:~/
ssh root@jhx.app "sudo k3s ctr -n k8s.io images import ~/${FILE_NAME}"

kubectl --context jhx-app apply -f k8s/namespace.yaml
kubectl --context jhx-app apply -f k8s/

#!/usr/bin/env bash

BASE_REGISTRY="gitlab-registry.isae-supaero.fr/dragoon/komodo"
MIRROR_REGISTRY="ghcr.io/dragoon-rs/dragoon/komodo"

revision="$1"

sd "image: \"$BASE_REGISTRY:.*\""   "image: \"$BASE_REGISTRY:$revision\""   .gitlab-ci.yml
sd "image: \"$MIRROR_REGISTRY:.*\"" "image: \"$MIRROR_REGISTRY:$revision\"" .github/workflows/ci.yml

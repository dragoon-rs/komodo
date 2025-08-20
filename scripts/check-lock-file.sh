#!/usr/bin/env bash

[[ -n "$(git status --short --no-branch --no-show-stash Cargo.lock)" ]] && exit 1 || exit 0

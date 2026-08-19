#!/bin/sh
set -eu

qemu_revision=fa19879df1658f96ac07365fca8835b7decd6995
temporary_checkout=

cleanup() {
    if test -z "$temporary_checkout"; then
        return
    fi
    if ! test -d "$temporary_checkout" || test -L "$temporary_checkout" || \
       ! test -f "$temporary_checkout/.gobject-linter-qemu-temporary"; then
        echo "refusing to remove unexpected temporary checkout: $temporary_checkout" >&2
        return 1
    fi
    cleanup_target=$temporary_checkout
    temporary_checkout=
    rm -rf -- "$cleanup_target"
}

if test -z "${QEMU_SRC:-}"; then
    temporary_checkout=$(mktemp -d "${TMPDIR:-/tmp}/gobject-linter-qemu.XXXXXX")
    touch "$temporary_checkout/.gobject-linter-qemu-temporary"
    trap cleanup EXIT HUP INT TERM

    git -C "$temporary_checkout" init --quiet
    git -C "$temporary_checkout" fetch --quiet --depth 1 \
        https://gitlab.com/qemu-project/qemu.git "$qemu_revision"
    git -C "$temporary_checkout" checkout --quiet --detach FETCH_HEAD
    QEMU_SRC=$temporary_checkout
    export QEMU_SRC
fi

CCACHE_DISABLE=1 cargo test -p gobject-ast --features qemu \
    --test qemu_corpus -- --ignored --nocapture

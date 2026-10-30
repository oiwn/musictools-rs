# Local plugin deployment (macOS). `just deploy` builds a release CLAP bundle
# and symlinks it into the user CLAP folder, so later builds update the
# installed plugin in place. Restart or rescan the host to pick up a rebuild.
# CLAP only for now: VST3 export sits behind each plugin's `vst3` feature.

clap_dir := home_directory() / "Library/Audio/Plug-Ins/CLAP"
bundled := justfile_directory() / "target/bundled"

# List recipes.
default:
    @just --list

# Build a release CLAP bundle into target/bundled.
bundle plugin="ppc_bogdan":
    cargo nice-plug bundle {{plugin}} --release

# Symlink the CLAP bundle into the user CLAP folder (replaces an installed copy).
link plugin="ppc_bogdan":
    #!/usr/bin/env bash
    set -euo pipefail
    src="{{bundled}}/{{plugin}}.clap"
    dst="{{clap_dir}}/{{plugin}}.clap"
    if [ ! -e "$src" ]; then
        echo "missing $src — run: just bundle {{plugin}}" >&2
        exit 1
    fi
    mkdir -p "{{clap_dir}}"
    if [ -L "$dst" ]; then
        echo "linked   $dst -> $(readlink "$dst")"
        exit 0
    fi
    if [ -e "$dst" ]; then
        echo "replace  $dst (installed copy)"
        rm -rf "$dst"
    fi
    ln -s "$src" "$dst"
    echo "linked   $dst -> $src"

# Remove the symlink made by `link` (never touches a real copy).
unlink plugin="ppc_bogdan":
    #!/usr/bin/env bash
    set -euo pipefail
    dst="{{clap_dir}}/{{plugin}}.clap"
    if [ -L "$dst" ]; then rm "$dst" && echo "removed  $dst"; fi

# Build and link in one go.
deploy plugin="ppc_bogdan": (bundle plugin) (link plugin)

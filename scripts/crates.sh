#!/bin/sh
#
# go list changes since last crate upload, per crate
#
for cargo in */Cargo.toml; do
    subdir="${cargo%/Cargo.toml}"
    desc="$(git describe --long --tags --match "${subdir}-*" 2>/dev/null)"
    test "$desc" = "" && continue
    desc="${desc%-*}" # drop hash
    desc="${desc%-*}" # drop commits
    name="${desc%-*}"
    vers="${desc##*-}"

    echo
    echo "#"
    echo "# $name -- $vers"
    echo "#"
    git log --oneline $desc.. -- $subdir
done

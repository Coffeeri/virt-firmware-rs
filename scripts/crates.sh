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

    commits=$(git rev-list $desc.. -- $subdir | wc -l)
    test "$commits" = "0" && continue

    echo
    echo "#"
    echo "# $name -- $vers [$commits commits]"
    echo "#"
    git log --oneline $desc.. -- $subdir
done

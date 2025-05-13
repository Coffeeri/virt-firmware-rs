#!/bin/sh
#
# crate release + upload script
#

subdir="${1%/}"

if test ! -d "$subdir"; then
    echo "usage: $0 <subdir>"
    exit 1
fi

project="virtfw-${subdir}"
version="$(yq .package.version ${subdir}/Cargo.toml)"
echo "# project: $project"
echo "# version: $version"

tag="${subdir}-v${version}"
if git tag --list "$tag" | grep -q "$tag"; then
    echo "# ERROR: tag $tag exists"
    exit 1
fi

case "$subdir" in
    efi-apps)
        target="--target $(uname -m)-unknown-uefi"
        ;;
    *)
        target=""
        ;;
esac

echo "# checking package ..."
(set -x; cargo package --package "$project" $target)	|| exit 1

echo "# uploading package ..."
(set -x; cargo publish --package "$project" $target)	|| exit 1

echo "# tagging release ..."
(set -x; git tag "$tag")				|| exit 1
(set -x; git push gitlab "$tag")			|| exit 1

echo "# watching ci build ..."
(set -x; sleep 10; gitlab-ci)

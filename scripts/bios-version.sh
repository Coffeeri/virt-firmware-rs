#!/bin/sh
#
# auto-update compiled-in bios version info for mefisto
#

# config
file="mefisto/src/bin/mefisto.rs"

# use current date
major=$(date +%y)
minor=$(date +%m)
date=$(date +%m/%d/%Y)

# report + update
echo "# v${major}.${minor#0} - ${date} - updating $file"
sed -e "/BIOS_MAJOR/s/= [^;]\+/= ${major}/" -i "$file"
sed -e "/BIOS_MINOR/s/= [^;]\+/= ${minor#0}/" -i "$file"
sed -e "/BIOS_DATE/s|= [^;]\+|= \"${date}\"|" -i "$file"
git commit -m "mefisto: bios version update (v${major}.${minor#0})" $file

#!/bin/bash
release_dir=mpm-$1
rm -rf $release_dir/*
mkdir -p $release_dir
cp -v target/release/mpm $release_dir

if [ "$1" == "windows-latest" ]; then
  7z a -r $release_dir.zip $release_dir
else
  zip -r $release_dir.zip $release_dir
fi

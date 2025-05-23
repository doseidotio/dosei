#!/bin/bash
set -e

cd macos

xcodebuild -target Dosei -configuration Release

cd ..

rm ./target/Dosei*.dmg

create-dmg --overwrite --identity "Apple Development: Alvaro Molina (BHFW3S86WS)" macos/build/Release/Dosei.app ./target

mv ./target/Dosei*.dmg ./target/Dosei.dmg

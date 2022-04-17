#!/usr/bin/env bash

echo "Bundling themes..."
for dir in ./themes/*/; do
    dir=${dir%*/}
    echo -ne "| ${dir##*/}"
    pushd "$dir" >> /dev/null
    zip -vr "${dir##*/}.zip" . -x "*.DS_Store" >> /dev/null
    mv "./${dir##*/}.zip" ../ >> /dev/null
    echo " ✓"
    popd >> /dev/null
done
echo "Done"
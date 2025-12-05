#!/bin/bash

find . -name "*.rs" -type f -not -path "*/target/*" | while read file; do
    sed -i '' -E 's/[[:space:]]+\/\/[^"]*$//' "$file"
    
    sed -i '' -E '/^[[:space:]]*\/\//d' "$file"
    
    sed -i '' -e '/\/\*/,/\*\//d' "$file"
done

echo "clean down"

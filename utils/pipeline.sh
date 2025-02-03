#!/bin/bash

# Start the pipeline
pipeline="bril2json"

# Loop through all arguments and append them to the pipeline
for path in "$@"; do
    pipeline="$pipeline | $path"
done

# Add the final step of the pipeline
pipeline="$pipeline"

# Execute the constructed pipeline
eval $pipeline
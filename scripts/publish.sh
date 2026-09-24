#!/usr/bin/env bash
set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/release-common.sh"

execute=false
if [[ ${1-} == --execute ]]; then
    execute=true
    shift
fi
select_packages "$@"

# Checks and uploads use the same package selection and working tree.
scripts/prepublish.sh "$@"
if [[ $execute == true ]]; then
    cargo publish "${release_selection[@]}" --locked
else
    printf '%s\n' 'Dry run complete. Use --execute to upload these packages.'
fi

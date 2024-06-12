#!/bin/bash

SIMICS_DIR="${1}"

if [ -z "${SIMICS_DIR}" ]; then
    echo "Usage: $0 <simics-dir>"
    exit 1
fi

if [ ! -d "${SIMICS_DIR}" ]; then
    echo "Error: ${SIMICS_DIR} is not a directory"
    exit 1
fi

TEMPDIR=$(mktemp -d)

readarray -t SIMICS_BASE_VERSIONS < <(find "${SIMICS_DIR}" -type d -regex '.*/simics\-[0-9]+\.[0-9]+\.[0-9]+' | sort -V | xargs -I{} basename {})

for i in $(seq 1 $((${#SIMICS_BASE_VERSIONS[@]} - 1))); do
    PRE="${SIMICS_BASE_VERSIONS[$i-1]}"
    CUR="${SIMICS_BASE_VERSIONS[$i]}"
    diff -ruN "${SIMICS_DIR}/${PRE}/src/include" "${SIMICS_DIR}/${CUR}/src/include" > "${TEMPDIR}/${PRE}-${CUR}.patch"
done

echo "Done. Patches are in ${TEMPDIR}."
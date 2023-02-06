#!/bin/bash
export LIBTORCH=$(python3 -c 'import torch; from pathlib import Path; print(Path(torch.__file__).parent)')
export DYLD_LIBRARY_PATH=${LIBTORCH}/lib
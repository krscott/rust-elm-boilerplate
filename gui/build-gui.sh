#!bash

# exit when any command fails
set -e
trap 'echo ''; echo Error at $(basename "$0"):${LINENO}: $BASH_COMMAND' ERR

# set working directory to this script's directory
cd "${0%/*}"

elm make --optimize --output=elm-dist.js src/Main.elm

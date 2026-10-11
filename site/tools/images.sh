#!/bin/sh
# Régénère les images PNG du site (cartes sociales, favicon, icône Apple).
# Il faut un Chromium ou Chrome sans interface : CHROME=/chemin/vers/chrome sh tools/images.sh
set -eu
cd "$(dirname "$0")"
CHROME="${CHROME:-chromium}"
shot() { # page largeur hauteur sortie [options…]
  page=$1 w=$2 h=$3 out=$4
  shift 4
  "$CHROME" --headless --no-sandbox --hide-scrollbars --force-device-scale-factor=1 \
    "$@" --window-size="$w,$h" --screenshot="$PWD/../src/$out" "file://$PWD/$page" >/dev/null 2>&1
  echo "écrit : src/$out"
}
shot og-image.html 1200 630 og-image.png
shot icon-favicon.html 32 32 favicon-32.png --default-background-color=00000000
shot icon-apple.html 180 180 apple-touch-icon.png

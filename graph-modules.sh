#!/bin/bash
set -x

cargo modules graph > modules.dot && dot -Tpng modules.dot > modules.png \
  && feh modules.png

exit

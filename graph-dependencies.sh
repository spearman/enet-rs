#!/usr/bin/env bash

set -e
set -x

cargo update
cargo deps --no-transitive-deps > dependencies.dot \
  && dot -Tpng dependencies.dot > dependencies.png \
  && feh dependencies.png

exit

help:
  just --list

watch-server:
  cargo watch -q -c -w apps/tetrad-server -w tetrad-api-contract/ -x "run -p tetrad-server"

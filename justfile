server_dir := "apps/tetrad-server"
server_crates_dir := server_dir + "/crates"

help:
  just --list

watch-server:
  cargo watch -q -c -w "{{ server_dir }}/services/tetrad-app" \
    -w tetrad-api-contract/ \
    -w "{{ server_crates_dir }}/tetrad-core" \
    -w "{{ server_crates_dir }}/tetrad-web" \
    -x "run -p tetrad-app"

# setns-shell

Bring your shell into [Rootless Docker](https://docs.docker.com/engine/security/rootless/) containers with you!

Zsh:

```zsh
$(readelf --string-dump .interp $(which zsh) | awk '/ld-linux/ {print $NF}') --library-path ${module_path} --preload "$(find ${module_path} -iname "*.so" -printf "%p:")" $(which zsh)
module_path=./target/debug zmodload libsetns_shell; zcompile -ac /tmp/full.zwc; setns_shell <PID 1 of container> /tmp/full.zwc
# Then copy/paste output back into shell
```


# TODO
- wrapper script using source
- mount instead of setns into process
- fork to make setns process really in process ns
- move away from export println!
- bash support

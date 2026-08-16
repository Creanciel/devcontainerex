# devcontainerex

English | [日本語](./README.ja.md)

A wrapper for `devcontainer exec`

Docker containers created by the VS Code Dev Containers extension may record Windows-style UNC paths,
while the Docker CLI looks up containers by POSIX paths on the WSL side, so the lookup fails and the container cannot be started as is.
This wrapper supplements the `--id-label` option so the same command works unchanged.

## Install

```sh
./install.sh
```

### Loading

Make sure the install destination `~/.local/bin` is on your PATH.
Self-reference protection is built in, so it is enough that this wrapper is the first one invoked.

#### alias

The safest way to avoid function-name collisions

```sh
cat >> ~/.bash_aliases <<'EOF'
# devcontainerex
alias devcontainer=devcontainerex
EOF
```

#### If you use mise

When mise is enabled, it rewrites PATH at shell startup, so `~/.local/bin` may end up
behind mise-managed tools and the PATH loading order is not stable.
To work around this, load an overrides directory via the PATH setting (`_.path` under `[env]`) in mise's global config.

```sh
mkdir -p ~/.local/overrides
ln -s ~/.local/bin/devcontainerex ~/.local/overrides/devcontainer

cat >> ~/.config/mise/config.toml <<'EOF'
[env]
_.path = ["{{env.HOME}}/.local/overrides"]
EOF
```

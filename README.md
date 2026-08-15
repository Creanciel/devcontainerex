# devcontainerex

`devcontainer exec` のラッパー

VS Code の Dev Containers 拡張が作成した Docker コンテナは Windows 視点の UNC パスを記録することがあり、
Docker CLI は WSL 側の POSIX パスのコンテナを探索するため解決できずそのまま起動ができない。
`--id-label` のオプションを補うことでそのままのコマンドで起動できるようにする。

## Install

```sh
./install.sh

cat >> ~/.bash_aliases <<'EOF'
# devcontainerex
alias devcontainer=devcontainerex
EOF
```

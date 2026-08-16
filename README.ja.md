# devcontainerex

[English](./README.md) | 日本語

`devcontainer exec` のラッパー

VS Code の Dev Containers 拡張が作成した Docker コンテナは Windows 視点の UNC パスを記録することがあり、
Docker CLI は WSL 側の POSIX パスのコンテナを探索するため解決できずそのまま起動ができない。
`--id-label` のオプションを補うことでそのままのコマンドで起動できるようにする。

## Install

```sh
./install.sh
```

### 読み込み

インストール先の `~/.local/bin` に PATH が通す。
自己参照対策は入っているので一番にこのラッパーが呼び出されるようになっていればよい。

#### alias

関数名の衝突を避けるのに一番安全な手段

```sh
cat >> ~/.bash_aliases <<'EOF'
# devcontainerex
alias devcontainer=devcontainerex
EOF
```

#### mise を使っている場合

mise を有効化していると shell 起動時に mise が PATH を書き換えるため、
`~/.local/bin` が mise 管理のツールより後ろに回ることがあり PATH の読み込み順が安定しない。
そこで mise のグローバル config の PATH 設定(`[env]` の `_.path`)で override 用ディレクトリを読み込むようにする。

```sh
mkdir -p ~/.local/overrides
ln -s ~/.local/bin/devcontainerex ~/.local/overrides/devcontainer

cat >> ~/.config/mise/config.toml <<'EOF'
[env]
_.path = ["{{env.HOME}}/.local/overrides"]
EOF
```

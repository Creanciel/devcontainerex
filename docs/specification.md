# devcontainerex 仕様

`devcontainer` CLI のラッパー(Rust / WSL 前提)。

## 目的

VS Code の Dev Containers 拡張は、コンテナのラベル
`devcontainer.local_folder` に Windows 視点の UNC パス
(`\\wsl.localhost\...`)を記録することがある。WSL のシェルから
`devcontainer exec` すると POSIX パスとの文字列一致に失敗し
`Dev container not found` になる。

devcontainerex はコンテナに実際に付いているラベル値を読み取り、
`--id-label` として補って実体に渡すことでこれを解決する。
ラベル値の推測・構築は行わず、読んで横流しするだけ。
ラベルが POSIX パスの通常環境でも無害に動作する。

## コマンド仕様

```
devcontainerex [SUBCOMMAND] [OPTIONS...] [COMMAND...]
```

| 呼び出し | 扱い |
|---|---|
| `exec` を含む | `--id-label`(と必要なら `--workspace-folder`)を挿入して実体に渡す |
| それ以外(サブコマンドなしを含む) | 一切変換せず実体に素通し |

- サブコマンドの判定は「`--` より前で最初に現れる `exec` トークン」で行う
  (実体の yargs はオプション混在を許すため、`exec` は第1引数とは限らない)。
  オプション値が偶然 `exec` という文字列の場合は誤検知するが許容する
- `--help` / `--version` を含む呼び出し(`--` より前)は、コンテナ探索を
  せず素通しする。コンテナ内コマンドに `--help` を渡したい場合は
  `--` より後ろに置く(`devcontainerex exec -- cmd --help`)
- 実体のオプションは列挙・再定義しない。解釈するのは
  `--workspace-folder` / `-w`(値を覗く。`=` 形式も可)と
  `--id-label`(既にあれば足さない)のみで、他はすべて素通し
- 独自フラグ・設定ファイルは持たない

## exec 時の処理

1. ワークスペースを決定する
   (`--workspace-folder` 引数 → `git rev-parse --show-toplevel` →
   カレントディレクトリ、の優先順で `canonicalize()`)
2. `docker ps -q` → `docker inspect` で起動中コンテナを列挙する
3. `devcontainer.config_file` ラベル(POSIX パスで記録される)が
   ワークスペース配下にあるものを選ぶ。前方一致はディレクトリ境界で
   判定する(`/foo` が `/foobar` を拾わない)。
   `devcontainer.local_folder` ラベルがないコンテナは除外する
4. 該当 1 件のコンテナの `devcontainer.local_folder` 値を取得する
5. `exec` トークン直後に `--id-label devcontainer.local_folder=<値>` を挿入し、
   `--workspace-folder` が未指定なら併せて付与して実体を `exec()` する
   (これがないと `remoteUser` / `remoteEnv` 等が適用されない)

## 実体(devcontainer CLI)の解決

```
1. DEVCONTAINEREX_DEVCONTAINER_BIN 環境変数
2. PATH 走査(canonicalize して自分自身を除外)
```

自己除外により `alias devcontainer=devcontainerex` 下でも再帰しない。
見つからない場合は環境変数での指定を促すエラーを出す。

## 実行方式

`exec()` によるプロセス置き換えで実体を起動する。

- TTY・シグナル・終了コードがそのまま引き継がれる
- シグナルハンドラは設定しない(`SIG_IGN` とマスクは execve 後も残るため)
- 引数はシェルを介さず渡す(ラベル値のバックスラッシュ対策)

## エラー

エラーは `error.rs` の `enum DevContainerExError`
(`Display` / `std::error::Error` を自前実装、外部クレート不使用)に集約し、
`main` が `error: ` 付きで表示する。

| 状況 | 挙動 | exit |
|---|---|---|
| 該当コンテナ 0 件 | `devcontainer up` を促す | 2 |
| 該当コンテナ 複数 | 候補(名前 / config_file)を一覧表示 | 2 |
| 実体が見つからない | `DEVCONTAINEREX_DEVCONTAINER_BIN` の指定を促す | 2 |
| docker の失敗 | docker の stderr をそのまま見せる | 2 |
| 実体の起動失敗 | ENOENT / EACCES 等 | 127 |
| exec 成功後 | 終了コードは実体からシェルへ直接伝播 | - |

## 非目標

- `devcontainer up` の代替(正常動作しているため素通しのみ)
- ポートフォワーディング等の再実装
- Windows ネイティブ環境(WSL 前提。`exec()` も Unix 限定)

## 構成

`apps/devcontainerex/src`:

| ファイル | 役割 |
|---|---|
| `main.rs` | エントリポイント。コンテナ特定・引数挿入・`exec()` |
| `args.rs` | 引数解釈(`Args::parse` → `Exec` / `Passthrough`) |
| `error.rs` | エラー型 `DevContainerExError` |
| `workspace.rs` | ワークスペースの決定 |
| `docker.rs` | `docker ps` / `docker inspect` によるコンテナ列挙 |
| `resolver.rs` | 実体の解決 |

インストールはリポジトリルートの `install.sh`(musl ビルド →
`~/.local/bin/` に配置)。alias 登録は手動(README 参照)。

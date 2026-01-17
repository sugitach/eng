# 00-emacs-next-gen-editor: Implementation Tasks

## Phase 1: サブSPECの定義と推進

- [x] **SPEC: `01-process-rpc`** (完了)
    *   RPC通信基盤の実装。
- [x] **SPEC: `02-buffer-design`** (完了)
    *   Ropeデータ構造と分散アーキテクチャの調査・設計。
- [x] **SPEC: `03-buffer-impl-core`** (完了)
    *   CoreプロセスにおけるRopeベースのバッファ基本実装。

- [x] **SPEC: `04-buffer-properties`** (完了)
    *   **内容**: バッファ名、パス、変更フラグ、エンコーディング等のメタデータを実装しました。
- [x] **SPEC実行: `04-buffer-properties`**
    *   **ブランチ**: `impl/buffer-properties`

- [x] **SPEC: `05-agent-setup`** (完了)
    *   **内容**: `eng-agent` クレートの新設と、UI-Agent-Core間の基本的なルーティングを実装しました。
- [x] **SPEC実行: `05-agent-setup`**
    *   **ブランチ**: `impl/agent-setup`

- [ ] **SPEC: `06-agent-lifecycle`**
    *   **内容**: AgentのDaemonモード、複数起動制御、終了時のCoreクリーンアップ処理を実装します。
    *   **実行コマンド**: `/kiro:spec-init 06-agent-lifecycle "AgentのライフサイクルとDaemonモード..."`
- [ ] **SPEC実行: `06-agent-lifecycle`**
    *   **内容**: コマンドライン引数処理、プロセス間通信（Agent探索）、終了シグナル伝搬の実装。

- [ ] **SPEC: `07-buffer-display`**
    *   **内容**: Coreのバッファ内容をRPC経由で効率的にUIへ転送し、`iced` を用いて描画する機能を実装します。
    *   **実行コマンド**: `/kiro:spec-init 07-buffer-display "バッファ内容のUI表示機能の実装..."`
- [ ] **SPEC実行: `07-buffer-display`**
    *   **内容**: 表示用RPCの定義、UI側の描画ロジック実装。

- [ ] **SPEC: `08-file-operations`**
    *   **内容**: ディスク上のファイルを読み書きする機能 (`OpenFile`, `SaveFile`) を実装します。
    *   **実行コマンド**: `/kiro:spec-init 08-file-operations "ファイル読み込み・保存機能の実装..."`
- [ ] **SPEC実行: `08-file-operations`**
    *   **内容**: ファイルI/Oの実装、非同期読み込み、エラーハンドリング。

## Phase 2: 統合と最終化
... (以下略)
# 02-buffer-design: Architecture Design

## 1. バッファ管理手法の選定
調査の結果、本プロジェクトでは **Rope** データ構造を採用する。

### 1.1 採用理由: Rope (ropey crate)
- **Rustエコシステムとの親和性**: `ropey` クレートは成熟しており、Unicode対応、行管理、スライス操作などが高速。
- **分散アーキテクチャへの適合**: 不変（Immutable）なスナップショットを安価に作成できるため、UIスレッドとCoreスレッド間でのデータ共有や、非同期バックグラウンド保存などがロックフリーに近い形で行える。
- **CRDTへの拡張性**: 将来的にZedのようなCRDT (RGA) を導入する際も、ツリー構造ベースであるため移行や統合が比較的容易。

## 2. UI-Core間通信プロトコル設計

### 2.1 編集操作 (Edit Operations)
巨大なテキスト全体を毎回送るのではなく、**差分 (Delta)** のみを転送する。
- **Request**: `ApplyEdit { operation: Insert/Delete, range: Range, text: String }`
- **Response**: `BufferUpdate { new_version: u64, changes: Vec<Change> }`

### 2.2 状態同期 (State Synchronization)
- **Version Vector / Logical Clock**: 各操作にバージョン番号を付与し、順序関係を保証する。
- **Optimistic UI**: UIはユーザー入力を即座に反映し、Coreからの正規化された更新を受け取った時点で補正する（Rebase的な挙動）。

## 3. 処理中断 (Quit / Cancel) の設計

Emacsの `C-g` (quit) 相当の機能を実現するため、**協調的キャンセル (Cooperative Cancellation)** モデルを採用する。

### 3.1 キャンセル機構のアーキテクチャ
OSシグナルによる強制終了はRustのリソース管理（Drop）と相性が悪いため、Tokioのタスク管理機能を活用する。

1.  **Job Manager (Core側)**:
    - ユーザーからの重いリクエスト（検索、置換、ファイル読み込み）は全て `Job ID` を発行し、`tokio::spawn` でタスク化する。
    - 各タスクの `AbortHandle` をMapで管理する。
2.  **Control RPC (優先制御チャネル)**:
    - 通常の編集RPCとは独立した（あるいは優先度付きの）制御用RPCを用意する。
    - UIから `CancelJob(JobId)` または `CancelAll` を送信する。
3.  **Cooperative Check**:
    - CPUバウンドな処理（巨大ループ等）の内部には、定期的に `tokio::task::yield_now()` または `cancellation_token.is_cancelled()` のチェックを挿入する規約とする。これにより、スレッドをブロックし続けることを防ぎ、キャンセル要求が即座に処理されるようにする。

### 3.2 プロトコル定義案 (Proto)
```protobuf
service EditorService {
  // ... 既存のRPC ...

  // 重い処理を開始（JobIdを返す）
  rpc Search(SearchRequest) returns (SearchResponse); // 内部でストリーミングまたはJob化

  // 実行中の処理を中断
  rpc Cancel(CancelRequest) returns (CancelResponse);
}

message CancelRequest {
  optional string job_id = 1; // 省略時はカレントコンテキストの全処理を中断
}
```

## 4. バッファデータ構造の詳細設計 (Core)
`eng-core` クレート内に `buffer` モジュールを作成する。

```rust
pub struct Buffer {
    text: ropey::Rope,
    path: Option<PathBuf>,
    version: u64,
    // ...
}
```
# 05-agent-setup: Design

## 1. プロセス階層
```text
[User] -> eng-agent (Supervisor, Session Manager, Script Engine)
           |
           +-- eng-ui (Window 1 - Local)
           |
           +-- eng-ui (Window 2 - Local)
           |
           +-- eng-core (Buffer Host A - Local)
           |
           +-- eng-core (Buffer Host B - Remote via SSH)
           |
           +-- eng-core (Buffer Host C - Remote via SSH)
```

## 2. 実装方針
### 2.1 eng-agent クレート
- `cargo new --bin eng-agent` で作成。
- **Session Manager**: 接続中のUI（クライアント）および、複数のCore（ローカル/リモート）のリストを管理する。
- **Launcher**: `eng-ui` および `eng-core` を起動・接続する機能を持つ。

### 2.2 通信プロトコル (editor.v1.proto 拡張)
- **AgentService**:
    - `RegisterUI(UiInfo) -> Stream<UiEvent>`: UIがAgentに接続し、描画命令等を受け取る。
    - `Agent` は `Core` に対してクライアントとして振る舞い、各UIからのリクエストを適切なCoreへルーティングする。

### 2.3 起動フロー
1. `eng-agent` 起動。
2. `eng-agent` がデフォルトの `eng-core` (Local) を起動し、接続。
3. `eng-agent` が最初の `eng-ui` を起動。
4. `eng-ui` が `eng-agent` に接続し、Windowを表示。
5. ユーザーがリモートファイルを開く際、`eng-agent` がリモートの `eng-core` を起動・接続する。

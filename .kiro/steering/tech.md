# Technology Stack

## Architecture
UI、Agent、Coreの3層からなる分散型アーキテクチャを採用する。**Agentプロセスがシステムのエントリーポイント**となり、他のプロセスのライフサイクルを管理する。

1. **Agent (`eng-agent`)**: **[Entry Point]**
    - ユーザーが直接起動するプロセス。Supervisorとして機能し、**UIとCoreを子プロセスとして起動・監視する**。
    - セッション管理、設定、**スクリプト実行**、およびCoreへのルーティングを担当するミドルウェア。ローカルで動作する。
2. **UI (`eng-ui`)**:
    - ユーザー入力とレンダリングを担当する軽量なフロントエンド。Agentによって起動され、Agentに接続する。別プロセスとして動作し、Agentのクラッシュやハングアップの影響を受けない。
3. **Core (`eng-core`)**:
    - バッファ管理、ファイル操作、テキスト編集を担当するバックエンド。Agentによって起動される（リモートの場合はSSH経由等）。

## Development Language
- **Rust**: 高いパフォーマンス、メモリ安全性、並行処理の容易さ、現代的な開発エコシステムとの親和性を理由に採用。

## Extension Language
設計段階で選定要件（Requirement 8, 10）に基づき、Lua (mlua), Python (PyO3), または WebAssembly (Wasmtime) などを候補として検討し、最終決定する。

**実行環境の方針**:
スクリプトエンジンは、ローカルで動作する **Agentプロセス (`eng-agent`)** 内にホストする。
- **役割**: UIからの入力を解釈し、適切なCoreへコマンドを発行する。また、UIへの表示指示を行う。
- **メリット**: スクリプトがUIスレッドをブロックしないため、重い処理中もUIの応答性を維持できる。また、複数のCore（ローカル、リモートA、リモートB）を単一のスクリプト環境から透過的に操作できる。

## UI Frameworks
- **ターミナルUI**: `tui-rs`や`ratatui`などのRust製TUIライブラリを検討。
- **ネイティブGUI**: `egui`, `iced`, `tauri` (Webviewベース) などを検討。
- **WebブラウザUI**: `WebAssembly`と`React`/`Vue`などのJavaScriptフレームワークを組み合わせることを検討。

## Communication Protocols
フロントエンドとバックエンド間の通信、およびサービス間の通信に利用。
- **低遅延**: `named-pipe` (ローカルIPC)、`WebSocket` (ネットワーク経由)
- **高ス throughput**: `HTTP/2` (大容量データ転送)
- **セキュリティ**: `SSH` (リモート接続)
- **汎用**: `gRPC` (構造化データ、多言語対応)

## AI Agent Integration
外部のAIサービス（OpenAI GPT, Gemini, Copilotなど）との連携を想定。AI Agent Serviceとして独立させ、エディタコアからAPIを介して連携する。

## Emacs Lisp Translator
Emacs Lispの構文解析、抽象構文木 (AST) 変換、ターゲット拡張言語へのコード生成技術を検討。独立したツールまたはサービスとして提供される可能性がある。

## Common Commands
- `cargo build`: プロジェクトのビルド
- `cargo test`: テストの実行
- `cargo run`: アプリケーションの実行（**`eng-agent` を起動し、システム全体を立ち上げる**）

## Process Lifecycle
システムの起動と終了は `eng-agent` によって厳格に管理される。

1. **起動エントリーポイント**: `eng-agent` が常にコマンドラインからの第一起動ポイントとなる。
2. **プロセス生成**: Agentは必要に応じて `eng-ui` (ローカル) および `eng-core` (ローカル/リモート) を起動する。
3. **終了条件 (Standard Mode)**: デフォルトでは、**全てのUIプロセスが終了した時点**でAgentも終了する。
4. **Coreのクリーンアップ**: Agent終了時、管理下の**全てのCoreプロセス（ローカル・リモート問わず）を確実に終了させる**。Agentとの接続が失われたCoreは自己終了するなどの安全策も講じる。
5. **Daemon Mode**: `--daemon` オプションで起動した場合、UIが全て閉じてもAgentは常駐し続ける。別途終了コマンド (`eng-agent --stop` 等) で終了する。
6. **インスタンス制御**: 既にDaemonモードのAgentが起動している状態で `eng-agent` を実行した場合、それは「クライアント」として振る舞う。既存のAgentに「新しいUIの起動」を指示した後、即座に終了する。
7. **ブロッキング挙動**:
    - **Standard Mode**: コマンドラインはAgent終了までブロックされる。
    - **Daemon Mode**: 起動完了後、即座にバックグラウンド化し制御を返す。

## Environment Variables
- `EDITOR_CORE_PORT`: エディタコアサービスのポート番号
- `AI_AGENT_API_KEY`: AI Agentサービスへの認証キー
- `EDITOR_LANG`: UIの表示言語設定

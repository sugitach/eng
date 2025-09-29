# Technology Stack

## Architecture
UIとエディタコアを完全に分離した分散型アーキテクチャを採用。UIはターミナル、ネイティブGUI、Webブラウザなど多様な環境で動作可能。エディタコアはバックエンドサービスとして機能し、ファイルシステムサービス、プロセス管理サービス、拡張機能ホスト、AI Agentサービスと連携する。

## Development Language
- **Rust**: 高いパフォーマンス、メモリ安全性、並行処理の容易さ、現代的な開発エコシステムとの親和性を理由に採用。Rust製IDEであるZedのソースコードを積極的に流用し、特にテキストレンダリング、構文解析、ファイル操作などのコンポーネントを検討する。

## Extension Language
設計段階で選定要件（Requirement 8, 10）に基づき、JavaScript, Perl, Python, Ruby, Lua などを候補として検討し、最終決定する。Elisp互換言語の実装はコストが高く、現代的な言語の豊富なライブラリやコミュニティの恩恵を受けられないため、Elispからの脱却を図る。

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
- `cargo run`: アプリケーションの実行

## Environment Variables
- `EDITOR_CORE_PORT`: エディタコアサービスのポート番号
- `AI_AGENT_API_KEY`: AI Agentサービスへの認証キー
- `EDITOR_LANG`: UIの表示言語設定

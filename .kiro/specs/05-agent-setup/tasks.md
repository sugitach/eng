# 05-agent-setup: Implementation Tasks

## 1. クレートの新設
- [x] 1.1 `cargo new --bin eng-agent` を実行し、ワークスペースに追加
- [x] 1.2 `eng-agent/Cargo.toml` の依存関係設定 (tonic, tokio, clap, uuid等)

## 2. ライフサイクル管理の移管
- [x] 2.1 `ui/src/launcher.rs` のロジックを `eng-agent` へ移行・汎用化
- [x] 2.2 AgentがCoreを起動し、認証情報を保持するロジックの実装
- [x] 2.3 AgentがUIを起動するロジックの実装

## 3. 通信基盤の構築 (UI-Agent)
- [x] 3.1 `proto/editor.v1.proto` に `AgentService` 定義を追加（既存のEditorServiceを流用）
- [x] 3.2 UIプロセスを「Agentに接続するクライアント」として改修
- [x] 3.3 AgentプロセスにUIからの接続を受け付けるサーバー機能を実装

## 4. 動作検証
- [x] 4.1 起動シーケンスの検証: Agent起動 -> Core/UIが連鎖起動 -> 通信確立
- [x] 4.2 マルチUI検証: Agentから2つ目のUIを起動し、それぞれが独立して動作することを確認 (SPEC 06へ移管)

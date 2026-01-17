# 06-agent-lifecycle: Requirements

## 1. 起動モード
1. **通常起動 (Standard)**: `eng-agent` で起動。UIを1つ開き、そのUIが閉じたらAgentも終了する。
2. **Daemon起動**: `eng-agent --daemon` で起動。UIが開いていなくても常駐し、コマンドライン制御を即座に返す。

## 2. 複数起動制御
1. **シングルトン**: ユーザーごとに1つのメインAgent（DaemonまたはStandard）が存在する。
2. **クライアント動作**: 既にAgentがいる状態で `eng-agent` を叩くと、既存Agentに接続して「新しいUIを開く」命令を送り、自分は終了する。

## 3. 終了処理
1. **連動終了**: Agent終了時、管理下の全UI・Coreプロセスを確実に終了させる。
2. **Core自決**: Agentとの接続が切れたCoreは、タイムアウト後に自動終了する安全策を持つ。

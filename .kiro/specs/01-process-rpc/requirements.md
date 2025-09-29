# 01-process-rpc: Requirements Document

## Introduction
本ドキュメントは、SPEC「01-process-rpc」に関する要求を定義します。ここでのゴールは、UIとコアがそれぞれ独立したプロセスとして起動し、互いに通信できる技術的な基盤を確立することです。

## Requirements

### Requirement 1: プロジェクト構造
**Objective:** `cargo` を用いて、UIとコアの2つのバイナリを含む単一のRustプロジェクトをセットアップする。

#### Acceptance Criteria
1. WHEN `cargo run --bin ui` を実行したとき THEN UIプロセスが起動し、ウィンドウが表示される（中身は空でよい） SHALL。
2. WHEN `cargo run --bin core` を実行したとき THEN コアプロセスが起動し、待機状態に入る SHALL。

### Requirement 2: プロセスの起動とライフサイクル管理
**Objective:** UIプロセスが、自身の起動後にコアプロセスを独立した子プロセスとして起動し、ライフサイクルを管理できる。

#### Acceptance Criteria
1. WHEN UIプロセスが起動したとき THEN UIは、関連するコアプロセスを自動的に子プロセスとして起動する SHALL。
2. IF UIプロセスが正常または異常終了したとき THEN 起動したコアプロセスも追随して終了する SHALL。

### Requirement 3: RPC通信路の確立
**Objective:** gRPC（または他の選定されたRPCフレームワーク）を用いて、UIプロセスとコアプロセス間の双方向ストリーミング通信を確立する。

#### Acceptance Criteria
1. WHEN 通信路が確立したとき THEN UIはコアに「Hello, Core!」というメッセージをRPCで送信できる SHALL。
2. IF コアがUIからのメッセージを受信したとき THEN コアはUIに「Hello, UI!」という応答メッセージをRPCで返信できる SHALL。
3. IF プロセス間の通信が切断された場合 THEN 両プロセスはエラーを検知し、適切にログを出力して終了処理を行う SHALL。
4. WHEN アイドル状態が一定時間続いた場合 THEN gRPCのキープアライブ機能により、接続が正常に維持されていることを確認する仕組みを持つ SHALL。相手プロセスが応答しない場合は、接続エラーとして検知できる SHALL。

### Requirement 4: プロセス間の衝突回避
**Objective:** 複数のUIプロセスが同時に起動されても、それぞれの「UI-コア」ペアが互いに干渉することなく、独立して動作できるようにする。

#### Acceptance Criteria
1. WHEN 2つ以上のUIプロセスがほぼ同時に起動されたとき THEN それぞれが正常に自身のコアプロセスを起動し、独立した通信を確立できる SHALL。
2. IF コアプロセスが通信のためにTCPポートを使用する場合 THEN ポート番号は静的な値ではなく、実行時に動的に割り当てられる SHALL。
3. WHERE UIプロセスがコアプロセスと通信を開始する際 THEN UIは、自身が起動したコアプロセスが使用している通信チャネル情報（例: 動的に割り当てられたポート番号）を、他のプロセスと衝突しない安全な方法で取得できる機構を持つ SHALL。

### Requirement 5: スキーマ管理
**Objective:** UIとコア間のAPI仕様を、Protocol Buffersのスキーマ定義ファイル（`.proto`）で明確に定義し、一元管理する。

#### Acceptance Criteria
1. WHEN RPCの仕様を定義・変更するとき THEN プロジェクトルートの `proto/` ディレクトリ以下に配置された `.proto` ファイルを更新する SHALL。このファイルがAPI仕様の唯一の信頼できる情報源となる。
2. IF `.proto` ファイルが更新された場合 THEN `cargo build` 時に `tonic-build` 等を用いて、サーバーとクライアントの雛形コードが自動生成される SHALL。
3. FOR このエピックでは、`Handshake`サービスを含む最小限のスキーマファイルを `proto/editor.v1.proto` として作成する SHALL。

### Requirement 6: 接続認証
**Objective:** 起動されたコアプロセスが、意図しない他のプロセスからの接続を拒否し、セキュリティを確保する。

#### Acceptance Criteria
1. WHEN UIプロセスがコアプロセスを起動するとき THEN 両者間でのみ共有される認証情報（例: ワンタイムトークン）が生成され、安全な方法（例: 標準入力パイプ）でコアプロセスに渡される SHALL。
2. IF コアプロセスが外部からgRPC接続リクエストを受け取ったとき THEN 正当な認証情報を含まない接続は即座に拒否する SHALL。
3. WHERE 正当なUIプロセスが接続した場合 THEN 認証は成功し、その後の通信が許可される SHALL。

### Requirement 7: 分散型アーキテクチャと多様なUIサポート
**Objective:** 現代的な開発環境の要求に応えるため、UIとエディタコアの分離、および多様なUI環境への対応を実現すること。

#### Acceptance Criteria
1. WHEN 次世代エディタが起動するとき THEN UIとバッファ操作を完全に分離し、それぞれが別のホスト（ローカルホストを含む）で協調動作する構造を実現 SHALL。
2. IF ローカルのファイル操作を行うとき THEN 次世代エディタは通信経由で処理を行うようにUIとバッファ操作を分離 SHALL。フロントエンドとバックエンドの接続方法はnamed-pipe, ssh, http(websocketなど)テキストでデータをやりとりできるものならどれでも検討範囲内とし、セキュリティ、低遅延、スループットの順に優先する SHALL。また、状況によっては同一のバックエンドとの接続に状況に応じた複数の接続を使い分ける方策も考慮する SHALL。
3. WHILE ユーザーが次世代エディタを利用している間 THEN UIはターミナル(コンソール)、OS native window, web browserのいずれでも動作する SHALL。
4. WHERE UIが動作する際 THEN 次世代エディタはコンポーネント単位で責務をきちんと分離した内部構造を実現 SHALL。バックエンドは純粋なデータ操作とロジックに専念し、UI固有の処理はフロントエンドが担当する SHALL。ターミナル等、制限のあるUIの場合の表示は各UI側で対応し、代替の効かない機能で実現できない場合はUI側がバックエンドにエラーを返すなどの方策を考慮する SHALL。

### Requirement 8: リモート実行バイナリの動的注入と実行
**Objective:** リモートホストへのSSH接続時に、クライアント側から適切なアーキテクチャのエディタコアバイナリを自動的に展開・実行し、シームレスなリモート開発体験を提供すること。

#### Acceptance Criteria
1. WHEN ユーザーがリモートホストに接続するとき THEN UIクライアントはターゲットホストのOSとアーキテクチャに適したエディタコアバイナリをネットワーク経由で転送する SHALL。
2. IF リモートホストが対応する機能（例: Linuxの`memfd_create`）を持つ場合 THEN バイナリはディスク上に一時ファイルを作成せず、メモリ上で直接実行されることを原則とする SHALL。
3. WHERE リモートホストでエディタコアが起動した後 THEN UIクライアントはそのプロセスと確立されたプロトコルで通信を開始する SHALL。
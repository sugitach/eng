# Project Structure

## Root Directory Organization
- `.kiro/`: Kiro-style Spec Driven Development関連ファイル（specs, steering, commands）
- `src/`: ソースコード
    - `core/`: エディタコアサービスの実装
    - `ui/`: 各UIアプリケーションの実装（terminal, native, web）
    - `extensions/`: 拡張機能ホストおよび拡張機能関連の実装
    - `agents/`: AI Agentサービスの実装
    - `translator/`: Emacs Lispトランスレータの実装
    - `common/`: 共通ライブラリ、データモデル、プロトコル定義
- `docs/`: ドキュメント
- `tests/`: テストコード
- `Cargo.toml`: Rustプロジェクトの設定ファイル

## Subdirectory Structures
- `src/core/`:
    - `buffer/`: バッファ管理ロジック
    - `command/`: コマンドディスパッチ、キーバインド処理
    - `io/`: ファイルI/O処理
    - `data_model/`: 共通データモデル定義
- `src/ui/[ui_type]/`:
    - `main.rs`: 各UIアプリケーションのエントリポイント
    - `components/`: UIコンポーネント
    - `event_handler/`: ユーザーイベント処理
- `src/extensions/`:
    - `host/`: 拡張機能ホストの実装
    - `api/`: 拡張機能API定義
    - `runtimes/`: 各拡張言語のランタイム統合
- `src/agents/`:
    - `service/`: AI Agentサービスの実装
    - `integrations/`: 外部AIサービスとの連携モジュール
- `src/translator/`:
    - `elisp_parser/`: Emacs Lisp構文解析
    - `ast_converter/`: AST変換ロジック
    - `code_generator/`: ターゲット言語へのコード生成

## Code Organization Patterns
- **モジュール性**: 各コンポーネントは独立したモジュールとして設計し、明確なインターフェースを介して連携する。
- **レイヤー化**: UI層、エディタコア層、サービス層（ファイルシステム、プロセス、拡張機能、AI Agent）といった論理的なレイヤーに分割する。
- **データ駆動**: データの構造と流れを重視し、状態管理は一貫した方法で行う。

## File Naming Conventions
- Rustの標準的な命名規則に従う（snake_case for files/modules, PascalCase for types/traits）。
- UIコンポーネントは、その役割を明確に示す名前とする。

## Import Organization
- `use`宣言は、標準ライブラリ、クレート、ローカルモジュールの順にグループ化し、アルファベット順にソートする。

## Key Architectural Principles
- **疎結合**: 各コンポーネントは可能な限り疎結合に保ち、変更の影響範囲を最小限にする。
- **高凝集**: 各コンポーネントは単一の責任を持ち、関連する機能は一つのコンポーネント内に集約する。
- **拡張性**: 新しいUIタイプや拡張言語、AI Agentの追加が容易な設計とする。
- **テスト容易性**: 各コンポーネントは独立してテスト可能であるように設計する。
- **パフォーマンス**: Rustの特性を活かし、ゼロコスト抽象化や効率的なデータ構造を積極的に利用する。

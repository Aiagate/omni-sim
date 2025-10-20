# Repository Guidelines

## プロジェクト構造とモジュール整理
- `src/core/` はイベントスケジューラとシミュレーション制御の中枢です。
- `src/entities/`、`src/events/`、`src/systems/`、`src/config/` はデータ・イベント・Systemロジック・定数を分離しています。イベントは`events`、反復処理は`systems`に置き、循環依存を避けてください。
- `examples/` は実行可能なデモ、`docs/` は設計資料、`tests/` はpytest向けスイートの配置場所です。新しいドキュメントはフェーズ別フォルダに追加し、実装例は`examples/`で共有します。

## ビルド・テスト・開発コマンド
- `uv sync` で依存関係を同期し、ロックファイルどおりの環境を再現します。
- `uv run python examples/basic_combat.py` はPhase 1戦闘デモ、`uv run python examples/resource_combat.py` は資源管理を含むPhase 2デモの再生に使います。
- `uv run python main.py` で最小エントリーポイントを確認し、`uv run python -m pytest` でテスト一式を実行します。対象を絞る際は`-k`オプションを活用してください。

## コーディングスタイルと命名規約
- PythonコードはPEP 8準拠・4スペースインデント・型ヒント必須。関数・変数は`snake_case`、クラスは`PascalCase`で統一します。
- コメント、docstring、ログ出力はすべて日本語で簡潔に記述し、挙動変更時は関連コメントも必ず更新します。
- 新規イベントクラスは`tick`と`priority`を明示する`__repr__`を実装し、シリアライズ可能な属性のみを保持してください。

## テストガイドライン
- テストは`tests/`直下に`test_<feature>.py`として追加し、共通フィクスチャは必要に応じて`tests/conftest.py`を新設して管理します。
- 主要イベントやSystem処理には正常系・制約違反系双方のpytestケースを用意し、再現手順はテスト関数のdocstringに残します。
- 新規機能はステートメントカバレッジ80%以上を目標とし、`uv run python -m pytest --cov=src`で確認した結果をPRに添付してください。

## コミットとプルリクエスト
- コミットメッセージは`feat: 燃料補給の上限を実装`のようにConventional Commits形式と短い日本語要約を組み合わせ、関連Issue番号は本文で`Refs #123`のように記載します。
- プルリクエストでは変更概要・検証手順・影響範囲・未解決事項を箇条書きで提示し、必要に応じて実行ログや出力例をコードブロックで共有してください。
- マージ前に`uv run python -m pytest`と主要デモシナリオの実行結果を報告し、レビュアーがローカルで再現できるよう追加設定や注意点をPR説明に記載します。

---
description: 実装内容をドキュメントに反映し、PRを発行できる状態に整える
---

# /pr-ready ワークフロー

新しい機能の実装後、ドキュメントを最新の状態に更新し、PRを発行するための手順。

## 手順

### 1. コードとドキュメントの乖離確認

追加・変更されたコンポーネントやシステムを特定し、ドキュメントの更新が必要な箇所をリストアップする。

```
view_file /home/dorothy/repos/omni-sim/src/components/mod.rs
view_file /home/dorothy/repos/omni-sim/src/systems/mod.rs
```

### 2. ドキュメントの更新

以下の優先順位でドキュメントを更新する。

1.  **docs/components.md**: フィールドや型の変更を反映。
2.  **docs/systems/*.md**: アルゴリズムの論理的な流れを更新。
3.  **docs/balance.md**: 数値定数や具体的な数式を更新（`src/config.rs` と一致させる）。
4.  **docs/architecture.md**: システム構成や実行順序（Mermaid）の変更を反映。
5.  **README.md**: 主要機能リストやディレクトリ構成の更新。

### 3. ドキュメント構成の整合性チェック

- [ ] `docs/README.md` に新しく追加したドキュメントが含まれているか。
- [ ] 相互リンクが切れていないか。
- [ ] プロポーザル（`docs/proposals/`）に新しい下書きが含まれている場合、それはマージ対象から外すか検討する。

### 4. PRの作成と公開

`docs/proposals/` や一時ファイルを除外してコミット・プッシュする。

```bash
# ブランチ作成
git checkout -b feature/your-feature-name

# 必要なファイルのみ追加 (proposalsなどを除外)
git add README.md AGENTS.md docs/ src/ Cargo.toml
git reset docs/proposals/

# コミット (Reset Authorが必要な場合は適宜)
git commit -m "docs: update documentation for [feature name]"

# プッシュとPR作成
git push -u origin feature/your-feature-name
gh pr create --title "Your PR Title" --body "PR description..."
```

## 注意点
- **AGENTS.md の遵守**: ドメインロジックへの表示用コードの混入がないか等を最終確認すること。
- **決定論性の維持**: ドキュメントに記述された数式が決定論を損なわないか再考すること。

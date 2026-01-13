# 02-buffer-design: Implementation Tasks

## 1. 既存エディタの調査
- [x] 1.1 EmacsのGap Buffer実装と限界の調査
- [x] 1.2 VS CodeのPiece Table実装の調査
- [x] 1.3 Zed/xi-editorのRope (SumTree) 実装の調査
- [x] 1.4 分散・協調編集におけるデータ同期アルゴリズム（OT vs CRDT）の調査

## 2. アーキテクチャ設計
- [x] 2.1 本プロジェクトにおけるバッファデータ構造の選定
- [x] 2.2 UI-Core間の通信プロトコル（編集情報の差分伝達手法）の設計
- [x] 2.3 Undo/Redo管理の設計方針策定
- [x] 2.4 行・列情報の高速計算手法の設計

## 3. レポート作成とレビュー
- [x] 3.1 調査レポートの作成 (design.mdに集約)
- [x] 3.2 アーキテクチャ設計書の完成
- [ ] 3.3 人間による設計レビューと承認

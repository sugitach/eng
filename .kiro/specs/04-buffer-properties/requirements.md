# 04-buffer-properties: Requirements

## 1. 機能要件
1. **基本メタデータ**: バッファ名 (name) とファイルパス (path) を保持できること。
2. **変更管理**: バッファが編集された際に modified フラグを適切に更新できること。
3. **読み取り専用制御**: read_only フラグによる編集制限ができること。
4. **形式保持**: encoding (デフォルトUTF-8) と line_ending (LF/CRLF) の情報を保持できること。

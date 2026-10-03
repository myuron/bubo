# bubo サンプルドキュメント

このファイルは、Markdown ビューアの表示確認に使うサンプルです。
見出し、強調、リスト、引用、コードブロック、表など、よく使う要素をひととおり含めています。
長い行の折り返しや全角文字の幅、ネストの深い構造、スクロールの確認にも使えます。

---

## 目次

1. [見出し](#見出し)
2. [インライン要素](#インライン要素)
3. [段落と改行](#段落と改行)
4. [リスト](#リスト)
5. [タスクリスト](#タスクリスト)
6. [引用](#引用)
7. [アラート](#アラート)
8. [コードブロック](#コードブロック)
9. [表](#表)
10. [水平線](#水平線)
11. [リンクと画像](#リンクと画像)
12. [HTML](#html)
13. [エスケープ](#エスケープ)
14. [長文](#長文)
15. [混在パターン](#混在パターン)

---

## 見出し

# 見出しレベル 1
## 見出しレベル 2
### 見出しレベル 3
#### 見出しレベル 4
##### 見出しレベル 5
###### 見出しレベル 6

Setext 形式の見出し 1
=====================

Setext 形式の見出し 2
---------------------

### 見出しの中の *強調* と `code`

#### 末尾に閉じ記号がある見出し ####

## インライン要素

これは *イタリック* で、これも _イタリック_ です。
これは **太字** で、これも __太字__ です。
これは ***太字かつイタリック*** で、これも ___太字かつイタリック___ です。
これは ~~取り消し線~~ で、こちらは ~一本チルダの取り消し線~ です。
これは `インラインコード` で、バッククォートを含む場合は `` `code` `` と書きます。

組み合わせの例:

- **太字の中に *イタリック* を入れる**
- *イタリックの中に **太字** を入れる*
- ~~取り消し線の中に **太字** と `code` を入れる~~
- **`太字のコード`**
- [**太字のリンク**](https://example.com)
- [`コードのリンク`](https://example.com)

英字とのまぜ書き: Rust の `Option<T>` は **null 安全** を実現するための *enum* です。

絵文字: 🦉 🚀 ✅ ❌ ⚠️ 📝 🎉

全角と半角の混在: ＡＢＣ abc １２３ 123 ｶﾀｶﾅ カタカナ

## 段落と改行

これは 1 つ目の段落です。
ソースでは改行していますが、ソフト改行なので表示上は 1 行につながります。

これは 2 つ目の段落です。行末に半角スペースを 2 つ置くと  
ハード改行になります。

バックスラッシュでもハード改行できます。\
この行は前の行と別の行に表示されます。

空行を複数入れても、段落の区切りは 1 つにまとまります。



この段落の前には空行が 3 つあります。

## リスト

### 箇条書き

- りんご
- バナナ
- みかん

* アスタリスクの箇条書き
* もう 1 つ

+ プラス記号の箇条書き
+ もう 1 つ

### 番号付きリスト

1. 最初の項目
2. 2 番目の項目
3. 3 番目の項目

### 途中の番号から始まるリスト

7. 7 番目
8. 8 番目
9. 9 番目
10. 10 番目（桁が増える）
11. 11 番目

### ネストしたリスト

- レベル 1
  - レベル 2
    - レベル 3
      - レベル 4
        - レベル 5
    - レベル 3 に戻る
  - レベル 2 に戻る
- レベル 1 に戻る

1. 番号付きの親
   1. 番号付きの子
   2. 番号付きの子
      - 箇条書きの孫
      - 箇条書きの孫
   3. 番号付きの子
2. 番号付きの親
   - 箇条書きの子
     1. 番号付きの孫
     2. 番号付きの孫

### 段落を含むリスト

- 最初の項目です。

  項目の中に 2 つ目の段落があります。インデントをそろえると同じ項目に属します。

- 2 番目の項目です。

  > 項目の中の引用です。

- 3 番目の項目です。

  ```sh
  echo "項目の中のコードブロック"
  ```

### 長い項目を含むリスト

- この項目はとても長いテキストを含んでいます。ターミナルの幅を超えたときに正しく折り返され、2 行目以降が箇条書き記号の位置ではなく本文の開始位置にそろうかを確認するためのものです。
- 短い項目
- Another very long item written in English to check how word wrapping behaves with ASCII text when the line exceeds the available width of the pane.

## タスクリスト

- [x] 設計を書く
- [x] テストを書く
- [ ] 実装する
- [ ] レビューを受ける
  - [x] セルフレビュー
  - [ ] ペアレビュー
- [ ] リリースする

1. [x] 番号付きのタスク
2. [ ] 番号付きの未完了タスク

## 引用

> これは引用です。
> 複数行にわたって書けます。

> 引用の中の段落 1 です。
>
> 引用の中の段落 2 です。

> ネストした引用:
>
> > 2 段目の引用
> >
> > > 3 段目の引用
> >
> > 2 段目に戻る
>
> 1 段目に戻る

> ### 引用の中の見出し
>
> - 引用の中のリスト
> - **太字** と *イタリック* と `code`
>
> ```rust
> fn quoted() -> &'static str {
>     "引用の中のコード"
> }
> ```

> 引用の中の長い行です。引用記号の後ろで折り返したときに、2 行目以降の先頭にも引用記号が付くかどうかを確認するために、わざと長く書いています。

## アラート

> [!NOTE]
> 補足情報です。読み飛ばしても問題ありませんが、知っておくと便利です。

> [!TIP]
> ヒントです。`just test` でテストをまとめて実行できます。

> [!IMPORTANT]
> 重要な情報です。必ず目を通してください。

> [!WARNING]
> 警告です。この操作には注意が必要です。

> [!CAUTION]
> 危険です。この操作は元に戻せません。

## コードブロック

### 言語指定あり

```rust
use std::collections::HashMap;

/// 単語の出現回数を数える
fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let counts = word_count("the quick brown fox jumps over the lazy dog");
    println!("{counts:?}");
}
```

```python
from dataclasses import dataclass


@dataclass
class Owl:
    name: str
    age: int

    def hoot(self) -> str:
        return f"{self.name}: ホーホー"


if __name__ == "__main__":
    print(Owl("bubo", 3).hoot())
```

```typescript
type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };

export async function fetchJson<T>(url: string): Promise<Result<T, string>> {
  try {
    const res = await fetch(url);
    return { ok: true, value: (await res.json()) as T };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}
```

```sh
#!/usr/bin/env bash
set -euo pipefail

for file in *.md; do
  echo "processing: ${file}"
done
```

```json
{
  "name": "bubo",
  "version": "0.1.0",
  "features": ["tree", "markdown", "scroll"],
  "nested": { "enabled": true, "count": 3 }
}
```

```toml
[package]
name = "bubo"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.29"
```

```diff
- let lines = app.file_lines(width).iter().take(height).cloned().collect();
+ let lines = app.file_view(width, height).to_vec();
```

### 言語指定なし

```
言語指定のないコードブロックです。
    インデントはそのまま保たれます。
```

### チルダのフェンス

~~~yaml
name: ci
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
~~~

### インデントによるコードブロック

    これは 4 つのスペースでインデントしたコードブロックです。
    fn indented() {}

### タブを含むコード

```go
func main() {
	for i := 0; i < 3; i++ {
		fmt.Println(i)
	}
}
```

### 長い行を含むコード

```text
この行はとても長く、ペインの幅を超えたときにコードブロックがどう表示されるかを確認するためのものです。ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789
```

### 空のコードブロック

```
```

## 表

### 基本の表

| 名前 | 種類 | 説明 |
| ---- | ---- | ---- |
| bubo | TUI | Markdown ビューア |
| ratatui | ライブラリ | TUI フレームワーク |
| crossterm | ライブラリ | ターミナル操作 |

### 揃え位置の指定

| 左揃え | 中央揃え | 右揃え |
| :----- | :------: | -----: |
| a | b | c |
| 長めのテキスト | 中央 | 123 |
| x | 中央に寄る値 | 4,567,890 |

### インライン要素を含む表

| 要素 | 例 |
| --- | --- |
| 太字 | **bold** |
| イタリック | *italic* |
| 取り消し線 | ~~strike~~ |
| コード | `code` |
| リンク | [example](https://example.com) |
| パイプ | `a \| b` |

### 列数の多い表

| # | 月 | 火 | 水 | 木 | 金 | 土 | 日 |
|---|----|----|----|----|----|----|----|
| 1 | ✅ | ✅ | ❌ | ✅ | ✅ | - | - |
| 2 | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | - |
| 3 | ❌ | ✅ | ✅ | ❌ | ✅ | - | ✅ |

### 長いセルを含む表

| キー | 説明 |
| --- | --- |
| `j` / `k` | ファイルツリーのカーソルを上下に移動します。 |
| `J` / `K` | 右ペインを 1 行ずつスクロールします。長いドキュメントを読むときに使います。 |
| `PageDown` / `PageUp` | 右ペインを表示の高さ分ずつスクロールします。 |
| `Space` | ディレクトリを開閉します。 |
| `Enter` | 選択中のファイルを右ペインに表示します。 |
| `q` | 終了します。 |

### 空のセルを含む表

| A | B | C |
| - | - | - |
| 1 |   | 3 |
|   | 2 |   |

## 水平線

3 種類の書き方があります。

---

***

___

## リンクと画像

### リンク

- インラインリンク: [Ratatui](https://ratatui.rs)
- タイトル付きリンク: [Rust](https://www.rust-lang.org "Rust 公式サイト")
- 参照リンク: [pulldown-cmark][cmark]
- 省略形の参照リンク: [crates.io][]
- 自動リンク: <https://github.com>
- メールアドレス: <owl@example.com>
- GFM の自動リンク: https://example.com/path?query=1
- 相対パス: [README](../README.md)
- アンカー: [表へ移動](#表)

[cmark]: https://github.com/pulldown-cmark/pulldown-cmark
[crates.io]: https://crates.io

### 画像

![フクロウのロゴ](https://example.com/owl.png)

![タイトル付きの画像](https://example.com/owl.png "フクロウ")

[![リンク付きの画像](https://example.com/badge.svg)](https://example.com)

## HTML

<details>
<summary>クリックで開く</summary>

折りたたまれた内容です。

</details>

<div align="center">
  <strong>中央揃えの HTML</strong>
</div>

インライン HTML: <kbd>Ctrl</kbd> + <kbd>C</kbd> でコピーします。H<sub>2</sub>O と x<sup>2</sup> も書けます。

<!-- これは HTML コメントです -->

## エスケープ

\*アスタリスクを強調にしない\*

\_アンダースコアを強調にしない\_

\# 見出しにしない

\- リストにしない

1\. 番号付きリストにしない

\`バッククォートをコードにしない\`

\[角括弧\]\(丸括弧\) をリンクにしない

バックスラッシュそのもの: \\

HTML 実体参照: &copy; &amp; &lt;tag&gt; &nbsp;&hearts;

## 長文

### 吾輩は猫である（冒頭）

吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。何でも薄暗いじめじめした所でニャーニャー泣いていた事だけは記憶している。吾輩はここで始めて人間というものを見た。しかもあとで聞くとそれは書生という人間中で一番獰悪な種族であったそうだ。

この書生というのは時々我々を捕えて煮て食うという話である。しかしその当時は何という考もなかったから別段恐しいとも思わなかった。ただ彼の掌に載せられてスーと持ち上げられた時何だかフワフワした感じがあったばかりである。

掌の上で少し落ちついて書生の顔を見たのがいわゆる人間というものの見始であろう。この時妙なものだと思った感じが今でも残っている。第一毛をもって装飾されべきはずの顔がつるつるしてまるで薬缶だ。その後猫にもだいぶ逢ったがこんな片輪には一度も出会わした事がない。

### Lorem ipsum

Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

### 区切りのない長い文字列

ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789

あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんアイウエオカキクケコサシスセソ

## 混在パターン

### 手順書の例

1. リポジトリをクローンします。

   ```sh
   git clone https://github.com/myuron/bubo.git
   cd bubo
   ```

2. 開発環境に入ります。

   > [!TIP]
   > Nix を使っている場合は `direnv allow` で自動的に入れます。

3. ビルドして実行します。

   ```sh
   just build
   ./target/debug/bubo
   ```

4. 次の項目を確認します。

   - [ ] ツリーが表示される
   - [ ] `Enter` でファイルが開ける
   - [ ] `J` / `K` でスクロールできる

### FAQ

**Q. 対応しているファイル形式は？**

A. Markdown です。それ以外のテキストファイルも表示できますが、Markdown として整形されます。

**Q. 画像は表示できますか？**

A. いいえ。`[image: ...]` のように代替テキストだけを表示します。

**Q. マウスは使えますか？**

A. 現時点ではキーボード操作のみです。

### 定義リスト風の書き方

**ソフト改行**
: 表示上は空白として扱われる改行です。

**ハード改行**
: 表示上も改行される改行です。

### 脚注風の書き方

脚注は有効化していないため、そのままのテキストとして表示されます[^1]。

[^1]: これは脚注の本文です。

### 数式風の書き方

インライン数式: $E = mc^2$

ブロック数式:

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$

### 変更履歴

| バージョン | 日付 | 内容 |
| :--------- | :--: | :--- |
| 0.1.0 | 2026-10-01 | 初回リリース |
| 0.1.1 | 2026-10-04 | 右ペインのスクロールに対応 |
| 0.2.0 | 未定 | ~~マウス対応~~ 検討中 |

---

###### おわり

最後まで読んでいただきありがとうございました。🦉

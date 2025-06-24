// src/components/msg.rs

/// UI内に表示される情報メッセージをAppのキューに追加するためのマクロ。
///
/// 使用例: `msg!(app, "Hello, world!");`
/// 使用例: `msg!(app, "Value: {}", some_value);`
///
/// 最初の引数として `&mut App` を受け取ります。
#[macro_export]
macro_rules! msg {
    ($app:expr, $($arg:tt)*) => {{
        // `format!`を使ってメッセージ文字列を構築
        let message = format!($($arg)*);
        // Appのadd_messageメソッドを呼び出してメッセージを追加
        $app.add_message($crate::app::MessageType::Info, message);
    }};
}

/// UI内に表示されるエラーメッセージをAppのキューに追加するためのマクロ。
///
/// 使用例: `emsg!(app, "Error: File not found!");`
/// 使用例: `emsg!(app, "Failed with code: {}", error_code);`
///
/// 最初の引数として `&mut App` を受け取ります。
#[macro_export]
macro_rules! emsg {
    ($app:expr, $($arg:tt)*) => {{
        // `format!`を使ってメッセージ文字列を構築
        let message = format!($($arg)*);
        // Appのadd_messageメソッドを呼び出してメッセージを追加
        $app.add_message($crate::app::MessageType::Error, message);
    }};
}

// マクロが外部で利用できるようにするために、これらはルートモジュールで公開される必要があります。
// 例: main.rs の先頭で `#[macro_use] extern crate <your_crate_name>;` を記述するか、
// crate::components::msg を pub use する。
// RataTuiのプロジェクト構造では、通常、`main.rs`で`use crate::msg;`のように直接インポートします。

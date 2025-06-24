// src/main.rs

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::app::{App, AppControlFlow, MessageType}; // AppControlFlowをインポート
use inedit::event_handler;
use inedit::ui;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::init();
    let tick_rate = Duration::from_millis(250); // UI更新レート

    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    _tick_rate: Duration, // tick_rateは現在event_handler内で直接使用されない
) -> io::Result<()> {
    // let mut last_tick = std::time::Instant::now(); // 不要になったので削除

    loop {
        // UIを描画
        terminal.draw(|f| ui::draw_ui(f, app))?;

        // イベントハンドラを呼び出し、アプリケーションの次の制御フローを取得
        let control_flow = event_handler::handle_event(app)?; // event_handlerから結果を取得

        match control_flow {
            AppControlFlow::Exit => {
                // アプリケーションを終了
                return Ok(());
            }
            AppControlFlow::TriggerSaveAndExit => {
                // 保存して終了
                match app.save_current_file() {
                    Ok(_) => return Ok(()), // 保存成功後、アプリ終了
                    Err(e) => {
                        app.add_message(MessageType::Error, format!("保存エラー: {}", e));
                        // エラーが発生してもアプリはすぐに終了せず、エラーメッセージを表示して続行する
                        // ユーザーが再度終了を試みるか、エラーを修正できるようにする
                    }
                }
            }
            AppControlFlow::TriggerDiscardAndExit => {
                // 変更を破棄して終了
                return Ok(());
            }
            AppControlFlow::Continue => {
                // アプリケーションを通常通り続行
                // 特に何もしない
            }
            AppControlFlow::ShowExitPopup => {
                // 終了ポップアップが表示されている状態なので、メインループは続行
                // UIは `app.exit_popup_state` に応じてポップアップを描画する
            }
        }
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let _r = n0_future::future::race(
        async move {
            let _r =
                client_terminal::terminal_ui::terminal_loop(&mut terminal).await;
            println!("* cli_chat_window closed: {:?}", _r);
        },
        async move {
            let _r = tokio::signal::ctrl_c().await;
            println!("* ctrl-c received");
        },
    )
    .await;

    ratatui::restore();
    println!("* shutting down...");

    println!("* shutdown OK");
    // std::process::exit(0);

    Ok(())
}

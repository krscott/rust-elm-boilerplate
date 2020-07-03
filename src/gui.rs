use serde::{de::DeserializeOwned, Serialize};
use std::env::current_exe;
use std::error::Error;
use std::fmt;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};
use std::thread;
use web_view::*;

fn gui_html() -> String {
    format!(
        r#"<!doctype html>
        <html>
        <head>
            <meta http-equiv="X-UA-Compatible" content="IE=edge">
            <meta charset="UTF-8">
            {styles}
        </head>
        <body>
            <!--[if lt IE 11]>
            <div class="ie-upgrade-container">
                <p class="ie-upgrade-message">Please, upgrade Internet Explorer to continue using this software.</p>
                <a class="ie-upgrade-link" target="_blank" href="https://www.microsoft.com/en-us/download/internet-explorer.aspx">Upgrade</a>
            </div>
            <![endif]-->
            <div id="elm"></div>
            {scripts}
        </body>
        </html>
		"#,
        styles = inline_style(include_str!("../gui/styles.css")),
        scripts = inline_script(include_str!("../gui/elm-dist.js"))
            + &inline_script(include_str!("../gui/app.js")),
    )
}

fn inline_style(s: &str) -> String {
    format!(r#"<style type="text/css">{}</style>"#, s)
}

fn inline_script(s: &str) -> String {
    format!(r#"<script type="text/javascript">{}</script>"#, s)
}

#[derive(Debug)]
pub enum GuiSendError {
    Serialize(serde_json::Error),
    Send(SendError<String>),
}

impl fmt::Display for GuiSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuiSendError::Serialize(e) => e.fmt(f),
            GuiSendError::Send(e) => e.fmt(f),
        }
    }
}

impl Error for GuiSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GuiSendError::Serialize(e) => Some(e),
            GuiSendError::Send(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum GuiRecvError {
    Deserialize(serde_json::Error),
    Recv(RecvError),
}

impl fmt::Display for GuiRecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuiRecvError::Deserialize(e) => e.fmt(f),
            GuiRecvError::Recv(e) => e.fmt(f),
        }
    }
}

impl Error for GuiRecvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GuiRecvError::Deserialize(e) => Some(e),
            GuiRecvError::Recv(e) => Some(e),
        }
    }
}

pub struct Gui {
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl Gui {
    pub fn new() -> Self {
        let (thread_tx, rx) = channel();
        let (tx, thread_rx) = channel();

        thread::spawn(move || {
            Self::gui_main(thread_tx, thread_rx);
        });

        Self { tx, rx }
    }

    pub fn recv<D: DeserializeOwned>(&self) -> Result<D, GuiRecvError> {
        let json = self.rx.recv().map_err(GuiRecvError::Recv)?;
        Ok(serde_json::from_str(&json).map_err(GuiRecvError::Deserialize)?)
    }

    pub fn send<S: Serialize>(&self, msg: S) -> Result<(), GuiSendError> {
        let json = serde_json::to_string(&msg).map_err(GuiSendError::Serialize)?;
        Ok(self.tx.send(json).map_err(GuiSendError::Send)?)
    }

    fn gui_main(tx: Sender<String>, rx: Receiver<String>) {
        let title = current_exe()
            .map(|pathbuf| pathbuf.as_path().to_string_lossy().to_string())
            .unwrap_or_else(|_| "".into());

        let webview = web_view::builder()
            .title(&title)
            .content(Content::Html(gui_html()))
            .resizable(true)
            .debug(true)
            .user_data(())
            .invoke_handler(|webview, arg| {
                // log::debug!("Message from GUI: {}", arg);
                tx.send(arg.to_owned()).unwrap();
                Self::render_noop(webview)
            })
            .build()
            .unwrap();

        let handle = webview.handle();

        thread::spawn(move || {
            while let Ok(state) = rx.recv() {
                handle
                    .dispatch(move |webview| Self::render(webview, state))
                    .unwrap();
            }
        });

        webview.run().unwrap();
    }

    fn render_noop(webview: &mut WebView<()>) -> WVResult {
        webview.eval("")
    }

    fn render(webview: &mut WebView<()>, json: String) -> WVResult {
        // log::debug!("Message to GUI: {}", json);
        let eval_js = format!("app.ports.fromRust.send({})", json);
        webview.eval(&eval_js)
    }
}

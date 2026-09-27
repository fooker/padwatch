use std::fmt::Write;

use anyhow::Result;
use matrix_sdk::ruma::api::client::message::send_message_event;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent, TextMessageEventContent};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::TransactionId;
use matrix_sdk::ruma::{OwnedRoomId, RoomId, ServerName};
use matrix_sdk::Client;
use similar::TextDiff;

use crate::pads::Pad;

pub struct Notifier {
    client: Client,
    room_id: OwnedRoomId,
}

impl Notifier {
    pub async fn connect(server_name: &ServerName, token: &str, room_id: &RoomId) -> Result<Self> {
        let client = Client::builder()
            .server_name(server_name)
            .handle_refresh_tokens()
            .build().await?;

       client.matrix_auth()
            .login_custom(
                "org.matrix.login.jwt",
                [("token".to_owned(), token.into())].into_iter().collect(),
            )?
            .initial_device_display_name("PadWatch Bot")
            .request_refresh_token()
            .await?;

        return Ok(Self {
            client,
            room_id: room_id.to_owned(),
        });
    }

    pub async fn notify(&self, pad: &Pad, orig: Option<&str>) -> Result<()> {
        let mut html = String::new();

        if orig.is_some() {
            write!(&mut html, "<b>Pad updated: </b>")?;
        } else {
            write!(&mut html, "<b>Pad created: </b>")?;
        };

        write!(&mut html, r###"<a href="{0}">{1}</a>"###,
               pad.link,
               pad.title)?;

        write!(&mut html, "<details><summary>Content:</summary><pre><code>")?;

        for hunk in TextDiff::from_lines(orig.unwrap_or(""), &pad.content)
            .unified_diff()
            .iter_hunks() {
            write!(&mut html, "{}\n", hunk)?;
        }

        write!(&mut html, "</code></pre></details>")?;

        let plain = format!("{}: {} \n  ⮡ {}",
                            if orig.is_some() { "Pad updated" } else { "Pad created" },
                            &pad.title,
                            &pad.link.to_string());

        let transaction = TransactionId::new();
        let content = AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::new(
                MessageType::Text(
                    TextMessageEventContent::html(plain, html))));
        let request = send_message_event::v3::Request::new(
            self.room_id.clone(),
            transaction.clone(),
            &content,
        )?;

        self.client.send(request).await?;

        return Ok(());
    }
}


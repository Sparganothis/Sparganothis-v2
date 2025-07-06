use crate::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::join_chat::{
        client_join_server_chat, ServerChatMessageContent, ServerChatRoomType,
    },
    user_identity::NodeIdentity,
};


#[derive(Debug, Clone)]
pub struct ClientApiManager {
    chat_controller: ChatController<ServerChatRoomType>,
    server_identity: NodeIdentity,
}

impl PartialEq for ClientApiManager {
    fn eq(&self, other: &Self) -> bool {
        self.chat_controller == other.chat_controller && self.server_identity == other.server_identity
    }
}

pub async fn connect_api_manager(
    mm: GlobalMatchmaker,
) -> anyhow::Result<ClientApiManager> {
    let (nodes, chat_controller) = client_join_server_chat(mm).await?;

    // TODO: make server_identity a mutex and update it with new server ids.
    Ok(ClientApiManager {
        chat_controller,
        server_identity: nodes[0],
    })
}

impl ClientApiManager {
    pub async fn send_login(&self) -> anyhow::Result<()> {
        let cc = self.chat_controller.clone();
        let sender = cc.sender();

        let request_message = ServerChatMessageContent::Request(
            super::join_chat::ServerMessageRequest::GuestLoginMessage {},
        );
        let receiver = cc.receiver().await;
        sender
            .direct_message(self.server_identity, request_message)
            .await?;

        while let Some(reply_message) = receiver.next_message().await {
            let reply = reply_message.message;

            let ServerChatMessageContent::Reply(reply) = reply else {
                anyhow::bail!("reply is not reply!");
            };
            let Ok(reply) = reply else {
                anyhow::bail!("error: {:?}", reply);
            };
            return Ok(());
        }

        anyhow::bail!("no more messages in chat!");
    }
}

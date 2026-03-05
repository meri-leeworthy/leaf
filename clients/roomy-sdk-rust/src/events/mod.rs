// Roomy event types and CBOR codec

use serde::{Deserialize, Serialize};

/// ULID wrapper type for serialization
pub type Ulid = String;

/// All Roomy event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Event {
    // Message events
    #[serde(rename = "space.roomy.message.createMessage.v0")]
    CreateMessage(CreateMessageEvent),

    #[serde(rename = "space.roomy.message.editMessage.v0")]
    EditMessage(EditMessageEvent),

    #[serde(rename = "space.roomy.message.deleteMessage.v0")]
    DeleteMessage(DeleteMessageEvent),

    // Space events
    #[serde(rename = "space.roomy.space.joinSpace.v0")]
    JoinSpace(JoinSpaceEvent),

    // Room events
    #[serde(rename = "space.roomy.room.createRoom.v0")]
    CreateRoom(CreateRoomEvent),

    #[serde(rename = "space.roomy.room.updateRoom.v0")]
    UpdateRoom(UpdateRoomEvent),

    #[serde(rename = "space.roomy.room.deleteRoom.v0")]
    DeleteRoom(DeleteRoomEvent),

    // Category events
    #[serde(rename = "space.roomy.category.createCategory.v0")]
    CreateCategory(CreateCategoryEvent),

    #[serde(rename = "space.roomy.category.updateCategory.v0")]
    UpdateCategory(UpdateCategoryEvent),

    #[serde(rename = "space.roomy.category.deleteCategory.v0")]
    DeleteCategory(DeleteCategoryEvent),

    // Page events
    #[serde(rename = "space.roomy.page.createPage.v0")]
    CreatePage(CreatePageEvent),

    #[serde(rename = "space.roomy.page.editPage.v0")]
    EditPage(EditPageEvent),

    #[serde(rename = "space.roomy.page.deletePage.v0")]
    DeletePage(DeletePageEvent),

    // Member events
    #[serde(rename = "space.roomy.member.addMember.v0")]
    AddMember(AddMemberEvent),

    #[serde(rename = "space.roomy.member.removeMember.v0")]
    RemoveMember(RemoveMemberEvent),

    #[serde(rename = "space.roomy.member.updateMemberRole.v0")]
    UpdateMemberRole(UpdateMemberRoleEvent),

    // Reaction events
    #[serde(rename = "space.roomy.reaction.addReaction.v0")]
    AddReaction(AddReactionEvent),

    #[serde(rename = "space.roomy.reaction.removeReaction.v0")]
    RemoveReaction(RemoveReactionEvent),

    // Read status events
    #[serde(rename = "space.roomy.read.markRead.v0")]
    MarkRead(MarkReadEvent),
}

/// Create message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageEvent {
    pub id: Ulid,
    pub room: Ulid,
    pub body: Content,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

/// Edit message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageEvent {
    pub id: Ulid,
    pub room: Ulid,
    pub message_id: Ulid,
    pub body: Content,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

/// Delete message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMessageEvent {
    pub id: Ulid,
    pub room: Ulid,
    pub message_id: Ulid,
}

/// Join space event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinSpaceEvent {
    pub id: Ulid,
    pub space_did: String,
}

/// Create room event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub label: String,
    pub kind: String, // "channel", "category", "thread", "page"
    #[serde(default)]
    pub parent: Option<Ulid>,
}

/// Update room event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoomEvent {
    pub id: Ulid,
    pub room: Ulid,
    #[serde(default)]
    pub label: Option<String>,
}

/// Delete room event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRoomEvent {
    pub id: Ulid,
    pub room: Ulid,
}

/// Create category event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub label: String,
    #[serde(default)]
    pub parent: Option<Ulid>,
}

/// Update category event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryEvent {
    pub id: Ulid,
    pub category: Ulid,
    #[serde(default)]
    pub label: Option<String>,
}

/// Delete category event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCategoryEvent {
    pub id: Ulid,
    pub category: Ulid,
}

/// Create page event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub room: Ulid,
    pub title: String,
    pub body: Content,
}

/// Edit page event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPageEvent {
    pub id: Ulid,
    pub page: Ulid,
    pub body: Content,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub snippet: Option<String>,
}

/// Delete page event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePageEvent {
    pub id: Ulid,
    pub page: Ulid,
}

/// Add member event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub user: String, // DID
    #[serde(default)]
    pub role: Option<String>,
}

/// Remove member event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveMemberEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub user: String, // DID
}

/// Update member role event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRoleEvent {
    pub id: Ulid,
    pub space: Ulid,
    pub user: String, // DID
    pub role: String,
}

/// Add reaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReactionEvent {
    pub id: Ulid,
    pub message: Ulid,
    pub emoji: String,
    pub user: String, // DID
}

/// Remove reaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveReactionEvent {
    pub id: Ulid,
    pub reaction: Ulid,
}

/// Mark read event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkReadEvent {
    pub id: Ulid,
    pub room: Ulid,
    pub user: String, // DID
    pub last_read: Ulid,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "mime_type")]
    pub mime_type: String,
    pub data: Vec<u8>,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum Attachment {
    #[serde(rename = "space.roomy.attachment.reply.v0")]
    Reply { target: Ulid },

    #[serde(rename = "space.roomy.attachment.embed.v0")]
    Embed { embed: serde_json::Value },
}

/// Event type discriminator for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Message creation event
    CreateMessage,
    /// Message edit event
    EditMessage,
    /// Message deletion event
    DeleteMessage,
    /// Space join event
    JoinSpace,
    /// Room creation event
    CreateRoom,
    /// Room update event
    UpdateRoom,
    /// Room deletion event
    DeleteRoom,
    /// Category creation event
    CreateCategory,
    /// Category update event
    UpdateCategory,
    /// Category deletion event
    DeleteCategory,
    /// Page creation event
    CreatePage,
    /// Page edit event
    EditPage,
    /// Page deletion event
    DeletePage,
    /// Add member event
    AddMember,
    /// Remove member event
    RemoveMember,
    /// Update member role event
    UpdateMemberRole,
    /// Add reaction event
    AddReaction,
    /// Remove reaction event
    RemoveReaction,
    /// Mark read event
    MarkRead,
}

impl Event {
    /// Get the event type discriminator
    pub fn event_type(&self) -> EventType {
        match self {
            Event::CreateMessage(_) => EventType::CreateMessage,
            Event::EditMessage(_) => EventType::EditMessage,
            Event::DeleteMessage(_) => EventType::DeleteMessage,
            Event::JoinSpace(_) => EventType::JoinSpace,
            Event::CreateRoom(_) => EventType::CreateRoom,
            Event::UpdateRoom(_) => EventType::UpdateRoom,
            Event::DeleteRoom(_) => EventType::DeleteRoom,
            Event::CreateCategory(_) => EventType::CreateCategory,
            Event::UpdateCategory(_) => EventType::UpdateCategory,
            Event::DeleteCategory(_) => EventType::DeleteCategory,
            Event::CreatePage(_) => EventType::CreatePage,
            Event::EditPage(_) => EventType::EditPage,
            Event::DeletePage(_) => EventType::DeletePage,
            Event::AddMember(_) => EventType::AddMember,
            Event::RemoveMember(_) => EventType::RemoveMember,
            Event::UpdateMemberRole(_) => EventType::UpdateMemberRole,
            Event::AddReaction(_) => EventType::AddReaction,
            Event::RemoveReaction(_) => EventType::RemoveReaction,
            Event::MarkRead(_) => EventType::MarkRead,
        }
    }

    /// Get the event ID
    pub fn id(&self) -> &str {
        match self {
            Event::CreateMessage(e) => &e.id,
            Event::EditMessage(e) => &e.id,
            Event::DeleteMessage(e) => &e.id,
            Event::JoinSpace(e) => &e.id,
            Event::CreateRoom(e) => &e.id,
            Event::UpdateRoom(e) => &e.id,
            Event::DeleteRoom(e) => &e.id,
            Event::CreateCategory(e) => &e.id,
            Event::UpdateCategory(e) => &e.id,
            Event::DeleteCategory(e) => &e.id,
            Event::CreatePage(e) => &e.id,
            Event::EditPage(e) => &e.id,
            Event::DeletePage(e) => &e.id,
            Event::AddMember(e) => &e.id,
            Event::RemoveMember(e) => &e.id,
            Event::UpdateMemberRole(e) => &e.id,
            Event::AddReaction(e) => &e.id,
            Event::RemoveReaction(e) => &e.id,
            Event::MarkRead(e) => &e.id,
        }
    }

    /// Get the space/room associated with this event (if any)
    pub fn room(&self) -> Option<&str> {
        match self {
            Event::CreateMessage(e) => Some(&e.room),
            Event::EditMessage(e) => Some(&e.room),
            Event::DeleteMessage(e) => Some(&e.room),
            Event::CreateRoom(e) => Some(&e.space),
            Event::CreatePage(e) => Some(&e.room),
            Event::AddReaction(e) => Some(&e.message),
            Event::MarkRead(e) => Some(&e.room),
            _ => None,
        }
    }
}

/// Generate a new ULID
pub fn new_ulid() -> Ulid {
    ulid::Ulid::new().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = Event::CreateMessage(CreateMessageEvent {
            id: new_ulid(),
            room: new_ulid(),
            body: Content {
                mime_type: "text/markdown".to_string(),
                data: b"Hello, world!".to_vec(),
            },
            attachments: vec![],
            extensions: serde_json::json!({}),
        });

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("createMessage"));
    }
}

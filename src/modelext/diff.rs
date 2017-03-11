use std::collections::HashSet;

use discord::model::{
    Message,
    MessageUpdate,
    MessageType,
    User,
    RoleId,
    Attachment,
};
use serde_json::Value;

pub trait Diff {
    type Other;
    type Output;

    fn diff(&self, other: &Self::Other) -> Vec<Self::Output>;
}

pub enum MessageUpdateDiff {
    Kind(Option<MessageType>, Option<MessageType>),
    Content(Option<String>, Option<String>),
    Nonce(Option<String>, Option<String>),
    Tts(bool, bool),
    Pinned,
    UnPinned,
    EditedTimestamp(Option<String>, Option<String>),
    MentionEveryone(bool, bool),
    MentionAdded(User),
    MentionRemoved(User),
    MentionRoleAdded(RoleId),
    MentionRoleRemoved(RoleId),
    AttachmentAdded(Attachment),
    AttachmentRemoved(Attachment),
    EmbedsAdded(Value),
    EmbedsRemoved(Value),
}

impl Diff for Message {
    type Other = MessageUpdate;
    type Output = MessageUpdateDiff;

    fn diff(&self, other: &MessageUpdate) -> Vec<Self::Output> {
        let mut res = Vec::new();
        if other.kind != Some(self.kind) {
            res.push(MessageUpdateDiff::Kind(Some(self.kind), other.kind));
        }
        if other.content.as_ref() != Some(&self.content) {
            res.push(MessageUpdateDiff::Content(Some(self.content.clone()), other.content.clone()));
        }
        if other.nonce != self.nonce {
            res.push(MessageUpdateDiff::Nonce(self.nonce.clone(), other.nonce.clone()));
        }
        if other.tts.unwrap_or_default() != self.tts {
            res.push(MessageUpdateDiff::Tts(self.tts, other.tts.unwrap_or_default()));
        }
        let otherpinned = other.pinned.unwrap_or_default();
        let selfpinned = self.pinned;
        if otherpinned & !selfpinned {
            res.push(MessageUpdateDiff::Pinned);
        } else if selfpinned & !otherpinned {
            res.push(MessageUpdateDiff::UnPinned);
        }
        if other.edited_timestamp != self.edited_timestamp {
            res.push(MessageUpdateDiff::EditedTimestamp(self.edited_timestamp.clone(), other.edited_timestamp.clone()));
        }
        if other.mention_everyone.unwrap_or_default() != self.mention_everyone {
            res.push(MessageUpdateDiff::MentionEveryone(self.mention_everyone, other.mention_everyone.unwrap_or_default()));
        }
        let empty = Vec::new();
        let mentions = other.mentions.as_ref().unwrap_or(&empty);
        let othermentions: HashSet<_> = mentions.iter().collect();
        let selfmentions: HashSet<_> = self.mentions.iter().collect();
        for &added in othermentions.difference(&selfmentions) {
            res.push(MessageUpdateDiff::MentionAdded(added.clone()));
        }
        for &removed in selfmentions.difference(&othermentions) {
            res.push(MessageUpdateDiff::MentionRemoved(removed.clone()));
        }
        let empty = Vec::new();
        let roles = other.mention_roles.as_ref().unwrap_or(&empty);
        let otherroles: HashSet<_> = roles.iter().collect();
        let selfroles: HashSet<_> = self.mention_roles.iter().collect();
        for &added in otherroles.difference(&selfroles) {
            res.push(MessageUpdateDiff::MentionRoleAdded(added.clone()));
        }
        for &removed in selfroles.difference(&otherroles) {
            res.push(MessageUpdateDiff::MentionRoleRemoved(removed.clone()));
        }
        let empty = Vec::new();
        let attachments = other.attachments.as_ref().unwrap_or(&empty);
        let otherattachments: HashSet<_> = attachments.iter().collect();
        let selfattachments: HashSet<_> = self.attachments.iter().collect();
        for &added in otherattachments.difference(&selfattachments) {
            res.push(MessageUpdateDiff::AttachmentAdded(added.clone()));
        }
        for &removed in selfattachments.difference(&otherattachments) {
            res.push(MessageUpdateDiff::AttachmentRemoved(removed.clone()));
        }
        let empty = Vec::new();
        let otherembeds = other.embeds.as_ref().unwrap_or(&empty);
        for added in  otherembeds.iter().filter(|&e| !self.embeds.contains(e)) {
            res.push(MessageUpdateDiff::EmbedsAdded(added.clone()));
        }
        for removed in self.embeds.iter().filter(|&e| !otherembeds.contains(e)) {
            res.push(MessageUpdateDiff::EmbedsRemoved(removed.clone()));
        }
        res
    }
}

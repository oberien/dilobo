use std::collections::{HashSet, HashMap};

use discord::model::{
    Message,
    MessageUpdate,
    MessageType,
    User,
    RoleId,
    Attachment,
    Member,
    ServerMemberUpdate,
    EmojiId,
    Emoji,
};
use serde_json::Value;

pub trait Diff {
    type Other;
    type Output;

    fn diff(&self, other: &Self::Other) -> Vec<Self::Output>;
}

pub enum MessageUpdateDiff {
    Kind(MessageType, MessageType),
    Content(String, String),
    Nonce(Option<String>, Option<String>),
    Tts(bool, bool),
    Pinned,
    UnPinned,
    EditedTimestamp(Option<String>, String),
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

impl MessageUpdateDiff {
    pub fn apply(&self, message: &mut Message) {
        match self {
            &MessageUpdateDiff::Kind(_, new) => message.kind = new,
            &MessageUpdateDiff::Content(_, ref new) => message.content = new.clone(),
            &MessageUpdateDiff::Nonce(_, ref new) => message.nonce = new.clone(),
            &MessageUpdateDiff::Tts(_, new) => message.tts = new,
            &MessageUpdateDiff::Pinned => message.pinned = true,
            &MessageUpdateDiff::UnPinned => message.pinned = false,
            &MessageUpdateDiff::EditedTimestamp(_, ref new) => message.edited_timestamp = Some(new.clone()),
            &MessageUpdateDiff::MentionEveryone(_, new) => message.mention_everyone = new,
            &MessageUpdateDiff::MentionAdded(ref user) => message.mentions.push(user.clone()),
            &MessageUpdateDiff::MentionRemoved(ref user) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                let pos = message.mentions.iter().position(|u| u.id == user.id).unwrap();
                message.mentions.swap_remove(pos);
            },
            &MessageUpdateDiff::MentionRoleAdded(role_id) => message.mention_roles.push(role_id),
            &MessageUpdateDiff::MentionRoleRemoved(role_id) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                let pos = message.mention_roles.iter().position(|id| *id == role_id).unwrap();
                message.mention_roles.swap_remove(pos);
            },
            &MessageUpdateDiff::AttachmentAdded(ref attachment) => message.attachments.push(attachment.clone()),
            &MessageUpdateDiff::AttachmentRemoved(ref attachment) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                let pos = message.attachments.iter().position(|a| a.id == attachment.id).unwrap();
                message.attachments.swap_remove(pos);
            },
            &MessageUpdateDiff::EmbedsAdded(ref val) => message.embeds.push(val.clone()),
            &MessageUpdateDiff::EmbedsRemoved(ref val) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                let pos = message.embeds.iter().position(|v| v == val).unwrap();
                message.embeds.swap_remove(pos);
            },
        }
    }
}

impl Diff for Message {
    type Other = MessageUpdate;
    type Output = MessageUpdateDiff;

    fn diff(&self, other: &MessageUpdate) -> Vec<Self::Output> {
        let mut res = Vec::new();
        if let Some(kind) = other.kind {
            if kind != self.kind {
                res.push(MessageUpdateDiff::Kind(self.kind, kind));
            }
        }
        if let Some(content) = other.content.as_ref() {
            if content != &self.content {
                res.push(MessageUpdateDiff::Content(self.content.clone(), content.clone()));
            }
        }
        if let Some(ref nonce) = other.nonce {
            if self.nonce.is_none() || nonce != self.nonce.as_ref().unwrap() {
                res.push(MessageUpdateDiff::Nonce(self.nonce.clone(), Some(nonce.clone())));
            }
        }
        if let Some(tts) = other.tts {
            if tts != self.tts {
                res.push(MessageUpdateDiff::Tts(self.tts, tts));
            }
        }
        if let Some(otherpinned) = other.pinned {
            if otherpinned && !self.pinned {
                res.push(MessageUpdateDiff::Pinned);
            } else if self.pinned && !otherpinned {
                res.push(MessageUpdateDiff::UnPinned);
            }
        }
        if let Some(ref ts) = other.edited_timestamp {
            if self.edited_timestamp.is_none() || ts != self.edited_timestamp.as_ref().unwrap() {
                res.push(MessageUpdateDiff::EditedTimestamp(self.edited_timestamp.clone(), ts.clone()));
            }
        }
        if let Some(mention_everyone) = other.mention_everyone {
            if mention_everyone != self.mention_everyone {
                res.push(MessageUpdateDiff::MentionEveryone(self.mention_everyone, mention_everyone));
            }
        }
        if let Some(ref mentions) = other.mentions.as_ref() {
            let othermentions: HashSet<_> = mentions.iter().collect();
            let selfmentions: HashSet<_> = self.mentions.iter().collect();
            for &added in othermentions.difference(&selfmentions) {
                res.push(MessageUpdateDiff::MentionAdded(added.clone()));
            }
            for &removed in selfmentions.difference(&othermentions) {
                res.push(MessageUpdateDiff::MentionRemoved(removed.clone()));
            }
        }
        if let Some(ref roles) = other.mention_roles.as_ref() {
            let otherroles: HashSet<_> = roles.iter().collect();
            let selfroles: HashSet<_> = self.mention_roles.iter().collect();
            for &added in otherroles.difference(&selfroles) {
                res.push(MessageUpdateDiff::MentionRoleAdded(added.clone()));
            }
            for &removed in selfroles.difference(&otherroles) {
                res.push(MessageUpdateDiff::MentionRoleRemoved(removed.clone()));
            }
        }
        if let Some(ref attachments) = other.attachments.as_ref() {
            let otherattachments: HashSet<_> = attachments.iter().collect();
            let selfattachments: HashSet<_> = self.attachments.iter().collect();
            for &added in otherattachments.difference(&selfattachments) {
                res.push(MessageUpdateDiff::AttachmentAdded(added.clone()));
            }
            for &removed in selfattachments.difference(&otherattachments) {
                res.push(MessageUpdateDiff::AttachmentRemoved(removed.clone()));
            }
        }
        if let Some(ref otherembeds) = other.embeds.as_ref() {
            for added in  otherembeds.iter().filter(|&e| !self.embeds.contains(e)) {
                res.push(MessageUpdateDiff::EmbedsAdded(added.clone()));
            }
            for removed in self.embeds.iter().filter(|&e| !otherembeds.contains(e)) {
                res.push(MessageUpdateDiff::EmbedsRemoved(removed.clone()));
            }
        }
        res
    }
}

pub enum MemberUpdateDiff {
    RoleAdded(RoleId),
    RoleRemoved(RoleId),
    NickChanged(Option<String>, Option<String>),
}

impl MemberUpdateDiff {
    pub fn apply(&self, member: &mut Member) {
        match self {
            &MemberUpdateDiff::RoleAdded(id) => member.roles.push(id),
            &MemberUpdateDiff::RoleRemoved(id) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                let pos = member.roles.iter().position(|rid| *rid == id).unwrap();
                member.roles.swap_remove(pos);
            },
            &MemberUpdateDiff::NickChanged(_, ref to) => member.nick = to.clone(),
        }
    }
}

impl Diff for Member {
    type Other = ServerMemberUpdate;
    type Output = MemberUpdateDiff;

    fn diff(&self, other: &Self::Other) -> Vec<Self::Output> {
        let mut res = Vec::new();
        let selfroles: HashSet<_> = self.roles.iter().cloned().collect();
        let otherroles: HashSet<_> = other.roles.iter().cloned().collect();
        for &added in otherroles.difference(&selfroles) {
            res.push(MemberUpdateDiff::RoleAdded(added));
        }
        for &removed in selfroles.difference(&otherroles) {
            res.push(MemberUpdateDiff::RoleRemoved(removed));
        }
        if self.nick != other.nick {
            res.push(MemberUpdateDiff::NickChanged(self.nick.clone(), other.nick.clone()));
        }
        res
    }
}

pub enum EmojisUpdateDiff {
    EmojiAdded(Emoji),
    EmojiRemoved(Emoji),
    NameChanged(EmojiId, String, String),
}

impl EmojisUpdateDiff {
    pub fn apply(&self, emojis: &mut HashMap<EmojiId, Emoji>) {
        match self {
            &EmojisUpdateDiff::EmojiAdded(ref emoji) => {
                // TODO: don't panic but return Result once error_chain is in place
                assert!(emojis.insert(emoji.id, emoji.clone()).is_none());
            },
            &EmojisUpdateDiff::EmojiRemoved(ref emoji) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                emojis.remove(&emoji.id).unwrap();
            },
            &EmojisUpdateDiff::NameChanged(id, _, ref new) => {
                // TODO: don't unwrap but return Result once error_chain is in place
                emojis.get_mut(&id).unwrap().name = new.clone();
            },
        }
    }
}

impl Diff for HashMap<EmojiId, Emoji> {
    type Other = Vec<Emoji>;
    type Output = EmojisUpdateDiff;

    fn diff(&self, others: &Self::Other) -> Vec<Self::Output> {
        let mut res = Vec::new();
        for other in others {
            let old = self.get(&other.id);
            if let Some(emoji) = old {
                if emoji.name != other.name {
                    res.push(EmojisUpdateDiff::NameChanged(emoji.id, emoji.name.clone(), other.name.clone()));
                }
                // TODO: handle roles attribute
            } else {
                res.push(EmojisUpdateDiff::EmojiAdded(other.clone()));
            }
        }
        let selfids: HashSet<_> = self.keys().cloned().collect();
        let otherids: HashSet<_> = others.iter().map(|e| e.id).collect();
        for removed_id in selfids.difference(&otherids) {
            // TODO: don't unwrap but return Result once error_chain is in place
            let removed = self.get(removed_id).unwrap();
            res.push(EmojisUpdateDiff::EmojiRemoved(removed.clone()));
        }
        res
    }
}

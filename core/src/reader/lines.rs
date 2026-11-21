use std::collections::HashMap;

const MINUTE_PREFIX_LENGTH: usize = "2026-11-21T10:00".len();

pub(super) struct OrderedMessage {
    pub(super) created_at: Box<str>,
    pub(super) id: Box<str>,
    pub(super) channel: u32,
    pub(super) author: u32,
    pub(super) progress: bool,
}

impl OrderedMessage {
    pub(super) fn minute(&self) -> &str {
        self.created_at
            .get(..MINUTE_PREFIX_LENGTH)
            .unwrap_or(&self.created_at)
    }

    pub(super) fn comes_before(&self, created_at: &str, id: &str) -> bool {
        (&*self.created_at, &*self.id) < (created_at, id)
    }
}

pub(super) enum Line {
    Message(usize),
    Fold { channel: u32, members: Vec<usize> },
}

impl Line {
    pub(super) fn latest(&self) -> Option<usize> {
        match self {
            Line::Message(position) => Some(*position),
            Line::Fold { members, .. } => members.last().copied(),
        }
    }

    pub(super) fn channel(&self, flow: &[OrderedMessage]) -> Option<u32> {
        match self {
            Line::Message(position) => flow.get(*position).map(|message| message.channel),
            Line::Fold { channel, .. } => Some(*channel),
        }
    }
}

pub(super) fn lines_of(flow: &[OrderedMessage]) -> Vec<Line> {
    let mut newest_first: Vec<Line> = Vec::new();
    let mut fold_growing_back_in_channel: HashMap<u32, usize> = HashMap::new();
    for (position, message) in flow.iter().enumerate().rev() {
        if !message.progress {
            fold_growing_back_in_channel.remove(&message.channel);
            newest_first.push(Line::Message(position));
        } else if let Some(Line::Fold { members, .. }) = fold_growing_back_in_channel
            .get(&message.channel)
            .and_then(|index| newest_first.get_mut(*index))
        {
            members.push(position);
        } else {
            fold_growing_back_in_channel.insert(message.channel, newest_first.len());
            newest_first.push(Line::Fold {
                channel: message.channel,
                members: vec![position],
            });
        }
    }
    newest_first.reverse();
    for line in &mut newest_first {
        if let Line::Fold { members, .. } = line {
            members.reverse();
        }
    }
    newest_first
}

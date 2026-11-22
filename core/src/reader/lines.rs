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
        self.order() < (created_at, id)
    }

    pub(super) fn comes_after(&self, created_at: &str, id: &str) -> bool {
        self.order() > (created_at, id)
    }

    fn order(&self) -> (&str, &str) {
        (&self.created_at, &self.id)
    }
}

pub(super) fn place_in_order(flow: &mut Vec<OrderedMessage>, arrived: Vec<OrderedMessage>) {
    let Some(earliest) = arrived
        .iter()
        .min_by(|first, second| first.order().cmp(&second.order()))
    else {
        return;
    };
    let first_to_move =
        flow.partition_point(|existing| existing.comes_before(&earliest.created_at, &earliest.id));
    flow.extend(arrived);
    if let Some(moved) = flow.get_mut(first_to_move..) {
        moved.sort_by(|first, second| first.order().cmp(&second.order()));
    }
}

pub(super) fn take_in_changes(flow: &mut [OrderedMessage], changed: Vec<OrderedMessage>) -> bool {
    let mut anything_moved = false;
    for message in changed {
        let place = flow.binary_search_by(|existing| existing.order().cmp(&message.order()));
        let Some(existing) = place.ok().and_then(|place| flow.get_mut(place)) else {
            continue;
        };
        let moves = existing.channel != message.channel
            || existing.author != message.author
            || existing.progress != message.progress;
        if moves {
            *existing = message;
            anything_moved = true;
        }
    }
    anything_moved
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

fn slot_of(channel: u32) -> Option<usize> {
    usize::try_from(channel).ok()
}

pub(super) fn lines_of(flow: &[OrderedMessage]) -> Vec<Line> {
    let channel_count = flow
        .iter()
        .filter_map(|message| slot_of(message.channel))
        .max()
        .map_or(0, |highest| highest.saturating_add(1));
    let mut fold_growing_back_in_channel: Vec<Option<usize>> = vec![None; channel_count];
    let mut newest_first: Vec<Line> = Vec::new();
    for (position, message) in flow.iter().enumerate().rev() {
        let growing =
            slot_of(message.channel).and_then(|slot| fold_growing_back_in_channel.get_mut(slot));
        let Some(growing) = growing else {
            newest_first.push(Line::Message(position));
            continue;
        };
        if !message.progress {
            *growing = None;
            newest_first.push(Line::Message(position));
        } else if let Some(Line::Fold { members, .. }) =
            (*growing).and_then(|index| newest_first.get_mut(index))
        {
            members.push(position);
        } else {
            *growing = Some(newest_first.len());
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

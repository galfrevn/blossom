pub struct Frame {
    pub id: Option<String>,
    pub data: Option<String>,
}

#[derive(Default)]
pub struct EventStreamReader {
    pending: Vec<u8>,
    frame_id: Option<String>,
    frame_data: Option<String>,
}

impl EventStreamReader {
    pub fn read(&mut self, bytes: &[u8]) -> Vec<Frame> {
        self.pending.extend_from_slice(bytes);
        let mut frames = Vec::new();
        let mut consumed = 0;
        while let Some(line_length) = self
            .pending
            .get(consumed..)
            .and_then(|rest| rest.iter().position(|byte| *byte == b'\n'))
        {
            let line_bytes = self
                .pending
                .get(consumed..consumed + line_length)
                .unwrap_or_default();
            let line = String::from_utf8_lossy(line_bytes);
            let line = line.strip_suffix('\r').unwrap_or(&line);
            if line.is_empty() {
                if self.frame_id.is_some() || self.frame_data.is_some() {
                    frames.push(Frame {
                        id: self.frame_id.take(),
                        data: self.frame_data.take(),
                    });
                }
            } else if !line.starts_with(':') {
                let (field, value) = line.split_once(':').unwrap_or((line, ""));
                let value = value.strip_prefix(' ').unwrap_or(value);
                match field {
                    "id" => self.frame_id = Some(value.to_owned()),
                    "data" => match self.frame_data.as_mut() {
                        Some(data) => {
                            data.push('\n');
                            data.push_str(value);
                        }
                        None => self.frame_data = Some(value.to_owned()),
                    },
                    _ => {}
                }
            }
            consumed += line_length + 1;
        }
        self.pending.drain(..consumed);
        frames
    }
}

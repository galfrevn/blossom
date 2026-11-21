use blossom::sse::{EventStreamReader, Frame};

fn read_all(reads: &[&[u8]]) -> Vec<Frame> {
    let mut reader = EventStreamReader::default();
    reads.iter().flat_map(|bytes| reader.read(bytes)).collect()
}

fn ids_and_data(frames: &[Frame]) -> Vec<(Option<&str>, Option<&str>)> {
    frames
        .iter()
        .map(|frame| (frame.id.as_deref(), frame.data.as_deref()))
        .collect()
}

const STREAM: &str = ": connected\n\nid: 812\nevent: board\ndata: {\"text\":\"déploiement échoué\"}\n\nid: 813\n\n: ping\n\n";

#[test]
fn frames_are_read_whole_however_the_stream_is_cut() {
    let whole = read_all(&[STREAM.as_bytes()]);
    let byte_by_byte: Vec<&[u8]> = STREAM.as_bytes().chunks(1).collect();
    let cut_byte_by_byte = read_all(&byte_by_byte);

    let expected = vec![
        (Some("812"), Some("{\"text\":\"déploiement échoué\"}")),
        (Some("813"), None),
    ];
    assert_eq!(ids_and_data(&whole), expected);
    assert_eq!(ids_and_data(&cut_byte_by_byte), expected);
}

#[test]
fn a_frame_is_not_read_until_its_blank_line_arrives() {
    let mut reader = EventStreamReader::default();

    let before_the_blank_line = reader.read(b"id: 812\ndata: {}\n");
    let after_it = reader.read(b"\n");

    assert!(before_the_blank_line.is_empty());
    assert_eq!(ids_and_data(&after_it), vec![(Some("812"), Some("{}"))]);
}

#[test]
fn lines_may_end_in_carriage_returns_and_data_may_span_lines() {
    let frames = read_all(&[b"id:812\r\ndata: first\r\ndata:second\r\n\r\n"]);

    assert_eq!(
        ids_and_data(&frames),
        vec![(Some("812"), Some("first\nsecond"))]
    );
}

#[test]
fn comments_and_unknown_fields_make_no_frame() {
    let frames = read_all(&[b": ping\n\nretry: 1000\n\nevent: board\n\n"]);

    assert!(frames.is_empty());
}

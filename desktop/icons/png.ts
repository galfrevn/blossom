import { crc32, deflateSync } from "node:zlib";

const signature = Buffer.from("89504e470d0a1a0a", "hex");
const channelsPerPixel = 4;
const bitDepth = 8;
const colorTypeWithAlpha = 6;
const headerBytes = 13;
const headerDepthOffset = 8;
const lengthBytes = 4;
const typeBytes = 4;
const checksumBytes = 4;

function chunk(type: string, data: Uint8Array): Uint8Array {
  const typeAndData = new Uint8Array(typeBytes + data.length);
  typeAndData.set(new TextEncoder().encode(type), 0);
  typeAndData.set(data, typeBytes);
  const bytes = new Uint8Array(
    lengthBytes + typeAndData.length + checksumBytes,
  );
  const view = new DataView(bytes.buffer);
  view.setUint32(0, data.length);
  bytes.set(typeAndData, lengthBytes);
  view.setUint32(lengthBytes + typeAndData.length, crc32(typeAndData));
  return bytes;
}

export function encodePng(size: number, pixels: Uint8Array): Uint8Array {
  const header = new Uint8Array(headerBytes);
  const headerView = new DataView(header.buffer);
  headerView.setUint32(0, size);
  headerView.setUint32(lengthBytes, size);
  header.set([bitDepth, colorTypeWithAlpha], headerDepthOffset);
  const rowLength = size * channelsPerPixel;
  const filteredRows = new Uint8Array(size * (rowLength + 1));
  for (let row = 0; row < size; row += 1) {
    filteredRows.set(
      pixels.subarray(row * rowLength, (row + 1) * rowLength),
      row * (rowLength + 1) + 1,
    );
  }
  const parts = [
    new Uint8Array(signature),
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(filteredRows)),
    chunk("IEND", new Uint8Array(0)),
  ];
  const png = new Uint8Array(
    parts.reduce((total, part) => total + part.length, 0),
  );
  let position = 0;
  for (const part of parts) {
    png.set(part, position);
    position += part.length;
  }
  return png;
}
